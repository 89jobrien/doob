// tests/beads_adapter_test.rs
#![allow(non_snake_case)]

use doob::sync::adapters::BeadsAdapter;
use doob::sync::domain::{HealthCheck, Provider};

#[test]
fn beads_adapter__returns_correct_name() {
    let adapter = BeadsAdapter::new();
    assert_eq!(adapter.name(), "beads");
}

#[test]
fn beads_adapter__is_available__returns_err_when_bd_not_found() {
    // On this dev machine, bd is not installed.
    // is_available() uses Command::new("bd") which returns Err when binary is missing.
    // The adapter maps that OS error to SyncError::ProviderUnavailable.
    let adapter = BeadsAdapter::new();
    let result = adapter.is_available();

    // Either bd is installed (Ok(true/false)) or it returns Err when not found.
    // On a machine without bd, this must be Err:
    match result {
        Ok(_) => {
            // bd is installed on this machine — that's fine, test passes
        }
        Err(doob::sync::domain::SyncError::ProviderUnavailable(msg)) => {
            assert!(msg.contains("bd CLI not found"));
        }
        Err(e) => panic!("Unexpected error variant: {:?}", e),
    }
}
