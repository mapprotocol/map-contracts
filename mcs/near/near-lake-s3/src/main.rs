use anyhow::Result;
use dotenv::dotenv;
use futures::join;
use crate::config::{init_redis_pusher, init_tracing, PROJECT_CONFIG, INDEXER};
use crate::indexer::stream::indexer_stream_from_s3;
use crate::pusher::redis::RedisPusher;

pub mod indexer;
pub mod pusher;
pub mod config;


#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    dotenv().ok();

    init_tracing();
    tracing::info!(target: INDEXER,".tracing is initialized");

    init_redis_pusher().await;
    tracing::info!(target: INDEXER,".redis pusher is initialized");

    indexer_stream_from_s3().await;

    Ok(())
}