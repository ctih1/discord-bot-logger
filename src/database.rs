use std::{collections::HashMap, fs::File};
use libsql::{Builder, Row};
use log::{info, error};
use tokio::sync::mpsc::{Sender, Receiver, channel};

#[derive(Debug)]
pub struct WriteJob {
    pub user_id: u64,
    pub time: u64,
    pub status: String,
    pub activity: String,
    pub activity_description: String
}

pub fn new_write_queue(buffer: usize) -> (Sender<WriteJob>, Receiver<WriteJob>) {
    channel(buffer)
}

pub async fn associate_usermame(id: u64, name: &str) {
    println!("Associating user {id} with username {name}");

    let db = Builder::new_local("data.db").build().await.unwrap();
    let conn = db.connect().unwrap();
    
    conn.execute(format!("
    CREATE TABLE IF NOT EXISTS users (
        id                      INTEGER PRIMARY KEY,
        username                MEDIUMTEXT
    )
    ").as_str(), ()).await.unwrap();

    let result: Result<u64, libsql::Error> = conn.execute(
        "INSERT OR REPLACE INTO users (id, username) VALUES (?1, ?2)",
        (id, name)
    ).await;
} 

pub async fn get_usernames(ids: Vec<u64>) -> HashMap<u64,String> {
    let db = Builder::new_local("data.db").build().await.unwrap();
    let conn = db.connect().unwrap();

    let mut results: HashMap<u64, String> = HashMap::new();

    conn.execute(format!("
    CREATE TABLE IF NOT EXISTS users (
        id                      INTEGER PRIMARY KEY,
        username                MEDIUMTEXT
    )
    ").as_str(), ()).await.unwrap();

    for id in ids.iter() {
        let mut query = conn.query(&format!("SELECT username FROM users WHERE id = {id}").as_str(),()).await.unwrap();
        if let Some(row) = query.next().await.unwrap() {
            results.insert(*id, row.get(0).unwrap());
        } else {
            println!("Could not find value for {id}");
            results.insert(*id, "unknown-user".to_string());
        }
    }
    return results
}

pub async fn get_data(page: &u64, user_id: Option<&str>, status: Option<&str>, activity: Option<&str>, activity_description: Option<&str>, time_lt: Option<&u64>, time_mt: Option<&u64>) -> libsql::Rows {
    let db = Builder::new_local("data.db").build().await.unwrap();
    let conn = db.connect().unwrap();
    let page_content_amount = 15;
    let min_id = (page -1) * ((page-1)*page_content_amount);
    
    let mut base_query = format!("SELECT * FROM tracking_data WHERE id IS NOT NULL");

    if let Some(user_id) = user_id {
        base_query += &format!(" AND user_id = {user_id}");
    }
    
    if let Some(status) = status {
        base_query += &format!(" AND status = '{status}'");
    }

    if let Some(activity) = activity {
        base_query += &format!(" AND activity = '{activity}'");
    }

    if let Some(activity_description ) = activity_description {
        base_query += &format!(" AND activity_description = '{activity_description}'");
    }

    if let Some(time_lt) = time_lt {
        base_query += &format!(" AND time < {time_lt}");
    }

    if let Some(time_mt) = time_mt {
        base_query += &format!(" AND time < {time_mt}");
    }

    base_query += &format!(" LIMIT {page_content_amount} OFFSET {min_id}");

    println!("Executing query {base_query}");

    return conn.query(&base_query, ()).await.unwrap();
}

pub async fn writer_task(mut rx: Receiver<WriteJob>) {
    let db = Builder::new_local("data.db").build().await.unwrap();
    let conn = db.connect().unwrap();
    
    conn.execute(format!("
    CREATE TABLE IF NOT EXISTS tracking_data (
        id                      INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id                 INTEGER,
        time                    INTEGER,
        status                  TINYTEXT,
        activity                MEDIUMTEXT,
        activity_description    MEDIUMTEXT
    )
    ").as_str(), ()).await.unwrap();

    while let Some(job) = rx.recv().await {
        println!("Performing write job");
        let result: Result<u64, libsql::Error> = conn.execute(
            "INSERT INTO tracking_data (user_id, time, status, activity, activity_description) VALUES (?1, ?2, ?3, ?4, ?5)",
            (job.user_id, job.time, job.status, job.activity, job.activity_description)
        ).await;
        if let Err(e) = result {
            error!("DB write failed {}", e);
        }
    }
}