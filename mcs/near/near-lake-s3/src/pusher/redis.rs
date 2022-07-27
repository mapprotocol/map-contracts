use std::thread::sleep;
use crate::config::PROJECT_CONFIG;
use anyhow::{anyhow, Result};
use std::time::Duration;
use futures::SinkExt;
use serde_json::Value;
use redis::{Client, RedisError, Commands, AsyncCommands};
use redis::aio::ConnectionManager;

pub struct RedisPusher {
    pub(crate) url: String,
    pub(crate) list_key: String,
    pub(crate) conn: ConnectionManager,
}

impl RedisPusher {
    pub async fn new(url: &String, list_key: &String) -> Result<Self> {
        let client = Client::open(url.clone())?;

        Ok(Self {
            url: url.clone(),
            list_key: list_key.clone(),
            conn: client.get_tokio_connection_manager().await?
        })
    }

    pub async fn lpush(&mut self, msg: String) {
        loop {
            let result = self.conn.lpush::<&str, &String, i32>(&self.list_key, &msg).await;
            if result.is_ok() {
                break;
            } else {
                println!("push msg {} to list {} failed with error {}, retry...", msg, &self.list_key, result.err().unwrap());
                sleep(Duration::from_secs(3))
            }
        }
    }

    pub async fn set(&mut self, key: &str, value: String) {
        loop {
            let result = self.conn.set::<&str, &String, String>(key, &value).await;
            if result.is_ok() {
                break;
            } else {
                println!("set key {} to value {} failed with error {}, retry...", key, value, result.err().unwrap());
                sleep(Duration::from_secs(3))
            }
        }
    }

    pub async fn get(&mut self, key: &str) -> Option<String> {
        loop {
            let result = self.conn.get::<&str, String>(key).await;
            if result.is_ok() {
                return Some(result.unwrap());
            } else {
                if let Ok(ret)  = self.conn.exists::<&str, i32>(key).await {
                    println!(" check if key {} exists: {}...", key, ret);
                    if ret == 0 {
                        return None;
                    }
                }
                println!("get value of key {} failed with error {}, retry...", key, result.err().unwrap());
                sleep(Duration::from_secs(3))
            }
        }
    }
}