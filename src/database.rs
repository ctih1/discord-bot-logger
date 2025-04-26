use std::fs::File;
use libsql::Builder;
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
        let result: Result<u64, libsql::Error> = conn.execute(
            "INSERT INTO tracking_data (user_id, time, status, activity, activity_description) VALUES (?1, ?2, ?3, ?4, ?5)",
            (job.user_id, job.time, job.status, job.activity, job.activity_description)
        ).await;
        if let Err(e) = result {
            error!("DB write failed {}", e);
        }
    }
}