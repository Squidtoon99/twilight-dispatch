use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use std::env;
use twilight_model::gateway::presence::{ActivityType, Status};

lazy_static! {
    pub static ref CONFIG: Config = {
        Config {
            rust_log: get_env("RUST_LOG"),
            bot_token: get_env("BOT_TOKEN"),
            shards_start: get_env_as("SHARDS_START"),
            shards_end: get_env_as("SHARDS_END"),
            shards_total: get_env_as("SHARDS_TOTAL"),
            shards_concurrency: get_env_as("SHARDS_CONCURRENCY"),
            shards_wait: get_env_as("SHARDS_WAIT"),
            clusters: get_env_as("CLUSTERS"),
            default_queue: get_env_as("DEFAULT_QUEUE"),
            resume: get_env_as("RESUME"),
            intents: get_env_as("INTENTS"),
            large_threshold: get_env_as("LARGE_THRESHOLD"),
            status: get_env_as("STATUS"),
            activity_type: get_env_as("ACTIVITY_TYPE"),
            activity_name: get_env("ACTIVITY_NAME"),
            log_channel: get_env_as("LOG_CHANNEL"),
            log_guild_channel: get_env_as("LOG_GUILD_CHANNEL"),
            state_enabled: get_env_as("STATE_ENABLED"),
            state_member: get_env_as("STATE_MEMBER"),
            state_member_ttl: get_env_as("STATE_MEMBER_TTL"),
            state_message: get_env_as("STATE_MESSAGE"),
            state_message_ttl: get_env_as("STATE_MESSAGE_TTL"),
            state_presence: get_env_as("STATE_PRESENCE"),
            state_old: get_env_as("STATE_OLD"),
            redis_host: get_env("REDIS_HOST"),
            redis_port: get_env_as("REDIS_PORT"),
        }
    };
}

#[derive(Clone, Debug)]
pub struct Config {
    pub rust_log: String,
    pub bot_token: String,
    pub shards_start: u64,
    pub shards_end: u64,
    pub shards_total: u64,
    pub shards_concurrency: u64,
    pub shards_wait: u64,
    pub clusters: u64,
    pub default_queue: bool,
    pub resume: bool,
    pub intents: u64,
    pub large_threshold: u64,
    pub status: Status,
    pub activity_type: ActivityType,
    pub activity_name: String,
    pub log_channel: u64,
    pub log_guild_channel: u64,
    pub state_enabled: bool,
    pub state_member: bool,
    pub state_member_ttl: u64,
    pub state_message: bool,
    pub state_message_ttl: u64,
    pub state_presence: bool,
    pub state_old: bool,
    pub redis_host: String,
    pub redis_port: u64,
}

fn get_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("Missing environmental variable: {}", name))
}

fn get_env_as<T: DeserializeOwned>(name: &str) -> T {
    let mut variable = get_env(name);

    simd_json::from_str(variable.as_mut_str())
        .or_else(|_| simd_json::from_str(format!(r#""{}""#, variable).as_mut_str()))
        .unwrap_or_else(|_| panic!("Invalid environmental variable: {}", name))
}
