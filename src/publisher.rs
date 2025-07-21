use {crate::config::Config, base58::ToBase58, redis::{AsyncCommands, RedisError}, solana_sdk::clock::Slot, std::time::Duration};

#[derive(Debug)]
pub struct Publisher {
    client: redis::Client,
    shutdown_timeout: Duration,
}

impl Publisher {
    pub fn new(client: redis::Client, config: &Config) -> Self {
        Self {
            client,
            shutdown_timeout: Duration::from_millis(config.redis.connection_timeout_ms),
        }
    }

    pub async fn update_account_info(
        &self,
        pubkey: &[u8],
        lamports: u64,
        slot: Slot,
    ) -> Result<(), RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let pubkey_str = pubkey.to_base58();

        let redis_key = format!("account:{}:{}", pubkey_str, slot);

        let _: () = conn
            .hset_multiple(redis_key, &[
                ("lamports", lamports.to_string()),
                ("slot", slot.to_string()),
            ])
            .await?;

        Ok(())
    }
}
