//! Test environment management

use crate::error::SprayError;
use elementsd::bitcoind::bitcoincore_rpc::RpcApi;
use elementsd::ElementsD;
use std::str::FromStr;

/// Test environment managing an Elements daemon
pub struct TestEnv {
    daemon: ElementsD,
    genesis_hash: musk::elements::BlockHash,
}

impl TestEnv {
    /// Create a new test environment with a fresh regtest daemon
    pub fn new() -> Result<Self, SprayError> {
        let mut conf = elementsd::Conf::new(None);

        // Increase initial free coins for testing
        let arg_pos = conf
            .0
            .args
            .iter()
            .position(|x| x.starts_with("-initialfreecoins="));

        match arg_pos {
            Some(i) => conf.0.args[i] = "-initialfreecoins=210000000000",
            None => conf.0.args.push("-initialfreecoins=210000000000"),
        };

        // Enable Simplicity
        conf.0.args.push("-evbparams=simplicity:-1:::");

        let daemon = ElementsD::with_conf(elementsd::exe_path().unwrap(), &conf)
            .map_err(|e| SprayError::DaemonError(e.to_string()))?;

        // Create wallet
        let create = daemon
            .client()
            .call::<serde_json::Value>("createwallet", &["wallet".into()])
            .map_err(|e| SprayError::RpcError(e.to_string()))?;

        if create.get("name").and_then(|v| v.as_str()) != Some("wallet") {
            return Err(SprayError::EnvironmentError(
                "Failed to create wallet".into(),
            ));
        }

        // Rescan blockchain
        let _rescan = daemon
            .client()
            .call::<serde_json::Value>("rescanblockchain", &[])
            .map_err(|e| SprayError::RpcError(e.to_string()))?;

        // Get genesis hash
        let genesis_str = daemon
            .client()
            .call::<serde_json::Value>("getblockhash", &[0u32.into()])
            .map_err(|e| SprayError::RpcError(e.to_string()))?;

        let genesis_hash = musk::elements::BlockHash::from_str(
            genesis_str
                .as_str()
                .ok_or_else(|| SprayError::EnvironmentError("Invalid genesis hash".into()))?,
        )
        .map_err(|e| SprayError::EnvironmentError(e.to_string()))?;

        Ok(Self {
            daemon,
            genesis_hash,
        })
    }

    /// Get a reference to the daemon
    pub fn daemon(&self) -> &ElementsD {
        &self.daemon
    }

    /// Get the genesis block hash
    pub fn genesis_hash(&self) -> musk::elements::BlockHash {
        self.genesis_hash
    }

    /// Generate blocks
    pub fn generate(&self, blocks: u32) -> Result<(), SprayError> {
        use elementsd::bitcoind::bitcoincore_rpc::RpcApi;

        let address = self
            .daemon
            .client()
            .get_new_address(None, None)
            .map_err(|e| SprayError::RpcError(e.to_string()))?;

        let address_str = address.assume_checked().to_string();

        self.daemon
            .client()
            .call::<serde_json::Value>(
                "generatetoaddress",
                &[blocks.into(), address_str.into()],
            )
            .map_err(|e| SprayError::RpcError(e.to_string()))?;

        Ok(())
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        // Daemon will be cleaned up automatically
    }
}

