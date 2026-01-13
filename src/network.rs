//! Network backend abstraction for spray
//!
//! Provides a unified interface over ephemeral regtest nodes and external nodes

use crate::client::ElementsClient;
use crate::env::TestEnv;
use crate::error::SprayError;
use musk::client::{ClientResult, NodeClient, Utxo};
use musk::elements::{Address, BlockHash, Transaction, Txid};
use musk::{Network, RpcClient};
use std::path::PathBuf;

/// Network backend abstraction
///
/// Supports both ephemeral local regtest nodes and external nodes via RPC
pub enum NetworkBackend {
    /// Ephemeral local regtest node (created on-demand, destroyed on drop)
    Ephemeral(TestEnv),
    /// External node via RPC (regtest, testnet, or liquid mainnet)
    External(RpcClient),
}

impl NetworkBackend {
    /// Get the genesis hash for this network
    ///
    /// # Errors
    ///
    /// Returns an error if the genesis hash cannot be retrieved from an external node.
    pub fn genesis_hash(&mut self) -> Result<BlockHash, SprayError> {
        match self {
            Self::Ephemeral(env) => Ok(env.genesis_hash()),
            Self::External(client) => client
                .genesis_hash()
                .map_err(|e| SprayError::RpcError(e.to_string())),
        }
    }

    /// Get address params for this network
    #[must_use]
    pub const fn address_params(&self) -> &'static musk::elements::AddressParams {
        match self {
            Self::Ephemeral(_) => &musk::elements::AddressParams::ELEMENTS,
            Self::External(client) => client.address_params(),
        }
    }
}

impl NodeClient for NetworkBackend {
    fn send_to_address(&self, addr: &Address, amount: u64) -> ClientResult<Txid> {
        match self {
            Self::Ephemeral(env) => {
                let client = ElementsClient::new(env.daemon());
                client.send_to_address(addr, amount)
            }
            Self::External(client) => client.send_to_address(addr, amount),
        }
    }

    fn get_transaction(&self, txid: &Txid) -> ClientResult<Transaction> {
        match self {
            Self::Ephemeral(env) => {
                let client = ElementsClient::new(env.daemon());
                client.get_transaction(txid)
            }
            Self::External(client) => client.get_transaction(txid),
        }
    }

    fn broadcast(&self, tx: &Transaction) -> ClientResult<Txid> {
        match self {
            Self::Ephemeral(env) => {
                let client = ElementsClient::new(env.daemon());
                client.broadcast(tx)
            }
            Self::External(client) => client.broadcast(tx),
        }
    }

    fn generate_blocks(&self, count: u32) -> ClientResult<Vec<BlockHash>> {
        match self {
            Self::Ephemeral(env) => {
                let client = ElementsClient::new(env.daemon());
                client.generate_blocks(count)
            }
            Self::External(client) => client.generate_blocks(count),
        }
    }

    fn get_utxos(&self, address: &Address) -> ClientResult<Vec<Utxo>> {
        match self {
            Self::Ephemeral(env) => {
                let client = ElementsClient::new(env.daemon());
                client.get_utxos(address)
            }
            Self::External(client) => client.get_utxos(address),
        }
    }

    fn get_new_address(&self) -> ClientResult<Address> {
        match self {
            Self::Ephemeral(env) => {
                let client = ElementsClient::new(env.daemon());
                client.get_new_address()
            }
            Self::External(client) => client.get_new_address(),
        }
    }
}

/// Create a network backend based on network type and optional config
///
/// # Errors
///
/// Returns an error if:
/// - Testnet is specified without a config file
/// - Config file cannot be read or parsed
/// - RPC client cannot be created
pub fn create_backend(
    network: Network,
    config: Option<PathBuf>,
) -> Result<NetworkBackend, SprayError> {
    match (network, config) {
        // Regtest without config: use ephemeral node
        (Network::Regtest, None) => {
            let env = TestEnv::new()?;
            Ok(NetworkBackend::Ephemeral(env))
        }
        // Regtest with config or testnet: use external node
        (_, Some(config_path)) => {
            let client = RpcClient::from_config_file(&config_path.to_string_lossy())
                .map_err(|e| SprayError::RpcError(e.to_string()))?;
            Ok(NetworkBackend::External(client))
        }
        // Testnet without config: error
        (Network::Testnet, None) => Err(SprayError::ConfigError(
            "Testnet requires --config <musk.toml> to specify node connection".into(),
        )),
        // Liquid mainnet
        (Network::Liquid, None) => Err(SprayError::ConfigError(
            "Liquid mainnet requires --config <musk.toml> to specify node connection".into(),
        )),
    }
}
