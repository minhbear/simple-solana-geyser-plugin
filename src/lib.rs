use {
    agave_geyser_plugin_interface::geyser_plugin_interface::{
        GeyserPlugin, GeyserPluginError, ReplicaAccountInfoV3, ReplicaAccountInfoVersions,
        ReplicaBlockInfoVersions, ReplicaEntryInfoVersions, ReplicaTransactionInfoVersions,
        SlotStatus,
    },
    log::{error, info},
    solana_sdk::clock::Slot,
    std::{fs::File, io::Read},
    tokio::runtime::Runtime,
};

mod config;
use config::*;
mod publisher;
use publisher::*;

#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = SimplePlugin::new();
    let plugin: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}

#[derive(Debug, Default)]
struct SimplePlugin {
    publisher: Option<Publisher>,
    runtime: Option<Runtime>,
}

impl GeyserPlugin for SimplePlugin {
    fn on_load(
        &mut self,
        config_file: &str,
        _is_reload: bool,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        solana_logger::setup_with_default("info");

        info!(
            "Loading plugin {:?} from config_file {:?}",
            self.name(),
            config_file
        );

        let mut file = File::open(config_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Config = serde_json::from_str(&contents).map_err(|err| {
            GeyserPluginError::ConfigFileReadError {
                msg: format!(
                    "The config file is not in the JSON format expected: {:?}",
                    err
                ),
            }
        })?;

        info!("Creating Redis client connection");

        let client = redis::Client::open(config.redis.url.clone()).unwrap();
        let publisher = Publisher::new(client, &config);
        let runtime = Runtime::new().map_err(|error| {
            error!("Failed to create tokio runtime: {error:?}");
            GeyserPluginError::Custom(Box::new(error))
        })?;

        self.publisher = Some(publisher);
        self.runtime = Some(runtime);

        Ok(())
    }

    fn on_unload(&mut self) {
        self.publisher = None;

        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }

    fn update_account(
        &self,
        account: ReplicaAccountInfoVersions,
        slot: Slot,
        _is_startup: bool,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        let info = Self::unwrap_update_account(account);
        let publisher = self.unwrap_publisher();

        let pubkey = info.pubkey;
        let lamports = info.lamports;

        let runtime = self.runtime.as_ref().expect("runtime is unavailable");
        runtime.block_on(async {
            publisher
                .update_account_info(pubkey, lamports, slot)
                .await
                .map_err(|e| GeyserPluginError::AccountsUpdateError { msg: e.to_string() })
        })?;

        Ok(())
    }

    fn notify_end_of_startup(
        &self,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        Ok(())
    }

    fn update_slot_status(
        &self,
        slot: Slot,
        parent: Option<u64>,
        status: &SlotStatus,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        Ok(())
    }

    fn notify_transaction(
        &self,
        transaction: ReplicaTransactionInfoVersions,
        slot: Slot,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        Ok(())
    }

    fn notify_entry(
        &self,
        entry: ReplicaEntryInfoVersions,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        Ok(())
    }

    fn notify_block_metadata(
        &self,
        blockinfo: ReplicaBlockInfoVersions,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        Ok(())
    }

    fn account_data_notifications_enabled(&self) -> bool {
        true
    }

    fn transaction_notifications_enabled(&self) -> bool {
        true
    }

    fn entry_notifications_enabled(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "SimplePlugin"
    }
}

impl SimplePlugin {
    pub fn new() -> Self {
        Self::default()
    }

    fn unwrap_publisher(&self) -> &Publisher {
        self.publisher.as_ref().expect("publisher is unavailable")
    }

    fn unwrap_update_account(account: ReplicaAccountInfoVersions) -> &ReplicaAccountInfoV3 {
        match account {
            ReplicaAccountInfoVersions::V0_0_1(_info) => {
                panic!(
                    "ReplicaAccountInfoVersions::V0_0_1 unsupported, please upgrade your Solana node."
                );
            }
            ReplicaAccountInfoVersions::V0_0_2(_info) => {
                panic!(
                    "ReplicaAccountInfoVersions::V0_0_2 unsupported, please upgrade your Solana node."
                );
            }
            ReplicaAccountInfoVersions::V0_0_3(info) => info,
        }
    }
}
