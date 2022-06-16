use crate::{
    cache,
    config::CONFIG,
    constants::{
        CONNECT_COLOR, DISCONNECT_COLOR, JOIN_COLOR, READY_COLOR, RESUME_COLOR,
    },
    utils::{log_discord, log_discord_guild},
};

use futures_util::{Stream, StreamExt};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};
use twilight_gateway::{Cluster, Event};

pub async fn outgoing(
    conn: &mut redis::aio::Connection,
    cluster: &Cluster,
    mut events: impl Stream<Item = (u64, Event)> + Send + Sync + Unpin + 'static,
) {
    let mut bot_id = None;

    while let Some((shard, event)) = events.next().await {
        let mut old = None;
        let shard = shard as usize;

        if CONFIG.state_enabled {
            if let Event::Ready(data) = &event {
                if bot_id.is_none() {
                    bot_id = Some(data.user.id);
                }
            }

            if let Some(bot_id) = bot_id {
                match timeout(
                    Duration::from_millis(10000),
                    cache::update(conn, &event, bot_id),
                )
                .await
                {
                    Ok(Ok(value)) => {
                        old = value;
                    }
                    Ok(Err(err)) => {
                        warn!("[Shard {}] Failed to update state: {:?}", shard, err);
                    }
                    Err(_) => {
                        warn!("[Shard {}] Timed out while updating state", shard);
                    }
                }
            }
        }

        match event {
            Event::GatewayHello(data) => {
                info!("[Shard {}] Hello (heartbeat interval: {})", shard, data);
            }
            Event::GatewayInvalidateSession(data) => {
                info!("[Shard {}] Invalid Session (resumable: {})", shard, data);
            }
            Event::Ready(data) => {
                info!("[Shard {}] Ready (session: {})", shard, data.session_id);
                log_discord(READY_COLOR, format!("[Shard {}] Ready", shard));
            }
            Event::Resumed => {
                if let Some(Ok(info)) = cluster.shard(shard as u64).map(|s| s.info()) {
                    info!(
                        "[Shard {}] Resumed (session: {})",
                        shard,
                        info.session_id().unwrap_or_default()
                    );
                } else {
                    info!("[Shard {}] Resumed", shard);
                }
                log_discord(RESUME_COLOR, format!("[Shard {}] Resumed", shard));
            }
            Event::ShardConnected(_) => {
                info!("[Shard {}] Connected", shard);
                log_discord(CONNECT_COLOR, format!("[Shard {}] Connected", shard));
            }
            Event::ShardConnecting(data) => {
                info!("[Shard {}] Connecting (url: {})", shard, data.gateway);
            }
            Event::ShardDisconnected(data) => {
                if let Some(code) = data.code {
                    let reason = data.reason.unwrap_or_default();
                    if !reason.is_empty() {
                        info!(
                            "[Shard {}] Disconnected (code: {}, reason: {})",
                            shard, code, reason
                        );
                    } else {
                        info!("[Shard {}] Disconnected (code: {})", shard, code);
                    }
                } else {
                    info!("[Shard {}] Disconnected", shard);
                }
                log_discord(DISCONNECT_COLOR, format!("[Shard {}] Disconnected", shard));
            }
            Event::ShardIdentifying(_) => {
                info!("[Shard {}] Identifying", shard);
            }
            Event::ShardReconnecting(_) => {
                info!("[Shard {}] Reconnecting", shard);
            }
            Event::ShardResuming(data) => {
                info!("[Shard {}] Resuming (sequence: {})", shard, data.seq);
            }
            Event::GuildCreate(data) => {
                if old.is_none() {
                    log_discord_guild(
                        JOIN_COLOR,
                        "Guild Join",
                        format!("{} ({})", data.name, data.id),
                    );
                }
            }
            _ => {}
        }
    }
}
