#![recursion_limit = "128"]
#![deny(clippy::all, nonstandard_style, rust_2018_idioms, unused, warnings)]
// https://github.com/rust-lang/rust-clippy/issues/7422
#![allow(clippy::nonstandard_macro_braces)]

use crate::{
    config::CONFIG,
    constants::{SESSIONS_KEY, SHARDS_KEY, STARTED_KEY},
    models::{ApiResult, FormattedDateTime, SessionInfo},
    utils::{get_clusters, get_queue, get_resume_sessions, get_shards},
};

use dotenv::dotenv;
use std::collections::HashMap;
use tokio::{join, signal::ctrl_c};
use tracing::{error, info};

mod cache;
mod config;
mod constants;
mod handler;
mod models;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let result = real_main().await;

    if let Err(err) = &result {
        error!("{:?}", err);
    }
    println!("{:?}", result.ok());
}

async fn real_main() -> ApiResult<()> {
    let redis = redis::Client::open(format!(
        "redis://{}:{}/",
        CONFIG.redis_host, CONFIG.redis_port
    ))?;

    let mut conn = redis.get_async_connection().await?;

    let shards = get_shards();
    let resumes = get_resume_sessions(&mut conn).await?;
    let resumes_len = resumes.len();
    let queue = get_queue();
    let (clusters, events) = get_clusters(resumes, queue).await?;

    info!("Starting up {} clusters", clusters.len());
    info!("Starting up {} shards", shards);
    info!("Resuming {} sessions", resumes_len);

    cache::set(&mut conn, STARTED_KEY, &FormattedDateTime::now()).await?;
    cache::set(&mut conn, SHARDS_KEY, &CONFIG.shards_total).await?;

    let mut conn_clone = redis.get_async_connection().await?;
    let mut conn_clone_two = redis.get_async_connection().await?;
    let clusters_clone = clusters.clone();
    tokio::spawn(async move {
        join!(
            cache::run_jobs(&mut conn_clone, clusters_clone.as_slice()),
            cache::run_cleanups(&mut conn_clone_two),
        )
    });

    for (cluster, events) in clusters.clone().into_iter().zip(events.into_iter()) {
        let cluster_clone = cluster.clone();
        tokio::spawn(async move {
            cluster_clone.up().await;
        });

        let mut conn_clone = redis.get_async_connection().await?;
        let cluster_clone = cluster.clone();
        tokio::spawn(async move {
            handler::outgoing(&mut conn_clone, &cluster_clone, events).await;
        });
    }

    ctrl_c().await?;

    info!("Shutting down");

    let mut sessions = HashMap::new();
    for cluster in clusters {
        for (key, value) in cluster.down_resumable().into_iter() {
            sessions.insert(
                key.to_string(),
                SessionInfo {
                    session_id: value.session_id,
                    sequence: value.sequence,
                },
            );
        }
    }

    cache::set(&mut conn, SESSIONS_KEY, &sessions).await?;

    Ok(())
}
