//! Unit tests for network backend abstraction

use musk::Network;
use spray::network::create_backend;

#[test]
fn test_create_backend_testnet_requires_config() {
    let result = create_backend(Network::Testnet, None);

    assert!(result.is_err(), "Testnet should require config");
    let err = result.err().expect("Expected error");
    assert!(
        err.to_string().contains("config") || err.to_string().contains("Config"),
        "Error should mention config requirement"
    );
}

#[test]
fn test_create_backend_liquid_requires_config() {
    let result = create_backend(Network::Liquid, None);

    assert!(result.is_err(), "Liquid mainnet should require config");
    let err = result.err().expect("Expected error");
    assert!(
        err.to_string().contains("config") || err.to_string().contains("Config"),
        "Error should mention config requirement"
    );
}

#[test]
fn test_create_backend_regtest_without_config_creates_ephemeral() {
    // This test requires elementsd, so we'll just verify the call doesn't panic on error handling
    // The actual ephemeral node creation is tested in integration tests
    let result = create_backend(Network::Regtest, None);

    // This may fail if elementsd is not installed, which is fine for unit tests
    // We just verify the function handles both success and failure gracefully
    match result {
        Ok(_) => {
            // Successfully created ephemeral node
        }
        Err(e) => {
            // Expected on systems without elementsd installed
            assert!(
                e.to_string().contains("daemon")
                    || e.to_string().contains("Daemon")
                    || e.to_string().contains("failed")
                    || e.to_string().contains("start"),
                "Error should relate to daemon startup: {}",
                e
            );
        }
    }
}

#[test]
fn test_create_backend_with_nonexistent_config() {
    use std::path::PathBuf;

    let config = Some(PathBuf::from("/nonexistent/path/musk.toml"));
    let result = create_backend(Network::Regtest, config);

    assert!(result.is_err(), "Should fail with nonexistent config file");
}
