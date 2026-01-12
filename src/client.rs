//! NodeClient implementation for ElementsD

use elementsd::bitcoind::bitcoincore_rpc::RpcApi;
use elementsd::ElementsD;
use musk::client::{ClientResult, NodeClient, Utxo};
use musk::elements::{encode::deserialize, hex::FromHex, Address, BlockHash, Transaction, Txid};
use std::str::FromStr;

/// NodeClient implementation wrapping ElementsD
pub struct ElementsClient<'a> {
    daemon: &'a ElementsD,
}

impl<'a> ElementsClient<'a> {
    pub fn new(daemon: &'a ElementsD) -> Self {
        Self { daemon }
    }
}

impl<'a> NodeClient for ElementsClient<'a> {
    fn send_to_address(&self, addr: &Address, amount: u64) -> ClientResult<Txid> {
        let addr_str = addr.to_string();
        // Convert satoshis to BTC (Elements uses BTC units)
        let amount_btc = amount as f64 / 100_000_000.0;

        let txid_str = self
            .daemon
            .client()
            .call::<serde_json::Value>("sendtoaddress", &[addr_str.into(), amount_btc.into()])
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?
            .as_str()
            .ok_or_else(|| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "Invalid txid response")))?
            .to_string();

        Txid::from_str(&txid_str)
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
    }

    fn get_transaction(&self, txid: &Txid) -> ClientResult<Transaction> {
        let tx_hex = self
            .daemon
            .client()
            .call::<serde_json::Value>("gettransaction", &[txid.to_string().into()])
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?
            .get("hex")
            .and_then(|v| v.as_str())
            .ok_or_else(|| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "Invalid transaction hex")))?
            .to_string();

        let tx_bytes = Vec::<u8>::from_hex(&tx_hex)
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        deserialize(&tx_bytes)
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
    }

    fn broadcast(&self, tx: &Transaction) -> ClientResult<Txid> {
        use musk::elements::encode::serialize_hex;

        let txid_str = self
            .daemon
            .client()
            .call::<serde_json::Value>("sendrawtransaction", &[serialize_hex(tx).into()])
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?
            .as_str()
            .ok_or_else(|| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "Invalid txid response")))?
            .to_string();

        Txid::from_str(&txid_str)
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
    }

    fn generate_blocks(&self, count: u32) -> ClientResult<Vec<BlockHash>> {
        let address = self
            .daemon
            .client()
            .get_new_address(None, None)
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let address_str = address.assume_checked().to_string();

        let result = self
            .daemon
            .client()
            .call::<serde_json::Value>(
                "generatetoaddress",
                &[count.into(), address_str.into()],
            )
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let hashes = result
            .as_array()
            .ok_or_else(|| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "Invalid block hash array")))?
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| BlockHash::from_str(s))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        Ok(hashes)
    }

    fn get_utxos(&self, _address: &Address) -> ClientResult<Vec<Utxo>> {
        // This is a simplified implementation
        // In a real implementation, you'd use listunspent or similar
        Ok(Vec::new())
    }

    fn get_new_address(&self) -> ClientResult<Address> {
        let addr = self
            .daemon
            .client()
            .get_new_address(None, None)
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        // Convert bitcoin::Address to elements::Address
        let addr_str = addr.assume_checked().to_string();
        Address::from_str(&addr_str)
            .map_err(|e| musk::ContractError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
    }
}

