mod database;

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;
use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::*;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

struct Data {}

struct Handler {
    tx:Sender<database::WriteJob>
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: serenity::Context, ready: Ready) {
        info!("Bot logged in to {}", ready.user.name);
    }

    async fn guild_create(&self, _ctx: serenity::Context, guild: Guild, _is_new: Option<bool>) {
        info!("Guild {} registered", guild.name);
    }

    async fn presence_update(&self, _ctx: serenity::Context, new_data: Presence) {
        if !new_data.guild_id.unwrap().get().to_string().eq(&env::var("SCAN_GUILD").unwrap()) {
            info!("Ignoring status update.");
        }

        info!("Presence update for {} arrived", new_data.user.name.unwrap());
        
        let activity = new_data
            .activities
            .first()
            .map(|a| a.name.as_str())
            .unwrap_or("Unknown");
        
        let activity_description = new_data
            .activities
            .first()
            .and_then(|a| a.details.as_deref())
            .unwrap_or("Unknown");

        let job = database::WriteJob {
            user_id: new_data.user.id.get(),
            time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            status: String::from(new_data.status.name()),
            activity: String::from(activity),
            activity_description: String::from(activity_description)
        };

        self.tx.send(job).await.unwrap();
    }
}


#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let (tx, rx) = database::new_write_queue(100);

    tokio::spawn(database::writer_task(rx));

    let handler = Handler {
        tx: tx.clone(),
    };

    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");
    
    let intents: GatewayIntents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILD_PRESENCES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build(); 


    let client = ClientBuilder::new(token,intents)
        .event_handler(handler)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}