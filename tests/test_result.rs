//! Unit tests for TestResult enum

use musk::elements::Txid;
use spray::TestResult;
use std::str::FromStr;

#[test]
fn test_result_is_success() {
    let txid = Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000")
        .expect("Valid txid");
    let result = TestResult::Success { txid };

    assert!(result.is_success());
    assert!(!result.is_failure());
}

#[test]
fn test_result_is_failure() {
    let result = TestResult::Failure {
        error: "Test failed".to_string(),
    };

    assert!(result.is_failure());
    assert!(!result.is_success());
}

#[test]
fn test_result_success_and_failure_mutually_exclusive() {
    let txid = Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000")
        .expect("Valid txid");
    let success = TestResult::Success { txid };
    let failure = TestResult::Failure {
        error: "error".to_string(),
    };

    // Success is not failure
    assert!(success.is_success() && !success.is_failure());

    // Failure is not success
    assert!(failure.is_failure() && !failure.is_success());
}

#[test]
fn test_result_clone() {
    let txid = Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000")
        .expect("Valid txid");
    let result = TestResult::Success { txid };
    let cloned = result.clone();

    assert!(cloned.is_success());
}

#[test]
fn test_result_debug() {
    let txid = Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000")
        .expect("Valid txid");
    let result = TestResult::Success { txid };
    let debug_str = format!("{:?}", result);

    assert!(debug_str.contains("Success"));
}
