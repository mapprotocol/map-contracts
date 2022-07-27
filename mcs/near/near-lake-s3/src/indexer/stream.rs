use crate::config::{init_lake_config, PROJECT_CONFIG, update_synced_block_height,
                    redis_publisher, INDEXER, init_redis_pusher};
use futures::StreamExt;
use serde_json::json;
use std::process::exit;
use std::time::Duration;
use near_lake_framework::near_indexer_primitives::views::{ExecutionOutcomeWithIdView, ExecutionStatusView};

pub async fn indexer_stream_from_s3() {
    let config = init_lake_config().await;

    let (_, stream) = near_lake_framework::streamer(config);

    let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
        .map(handle_streamer_message)
        .buffer_unordered(1usize);

    while let Some(_handle_message) = handlers.next().await {}
}

pub async fn handle_streamer_message(
    streamer_message: near_lake_framework::near_indexer_primitives::StreamerMessage,
) {
    println!("⛏ Block height {:?}", streamer_message.block.header.height);

    let mut publish = false;
    'outer: for shard in &streamer_message.shards {
        for tx_res in &shard.receipt_execution_outcomes {
            if is_valid_receipt(&tx_res.execution_outcome) {
                publish = true;
                break 'outer;
            }
        }
    }

    if !publish {
        return;
    }

    let json = json!(streamer_message).to_string();
    redis_publisher().lpush(json).await;
    update_synced_block_height(streamer_message.block.header.height).await;

    tracing::info!(
        target: INDEXER,
        "Save {} / shards {}",
        streamer_message.block.header.height,
        streamer_message.shards.len()
    );
}

pub fn is_valid_receipt(execution_outcome: &ExecutionOutcomeWithIdView) -> bool {
    match &execution_outcome.outcome.status {
        ExecutionStatusView::SuccessValue(_) => (),
        ExecutionStatusView::SuccessReceiptId(_) => (),
        _ => return false
    }

    PROJECT_CONFIG.mcs.eq(&execution_outcome.outcome.executor_id.to_string())
}
