use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[allow(dead_code)]
    libpath: String,

    pub redis: RedisConfig,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_ms: u64,

    #[serde(default)]
    pub database: i64,
}

fn default_max_connections() -> u32 {
    10
}

fn default_connection_timeout() -> u64 {
    5000
}
