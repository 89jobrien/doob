// tests/beads_adapter_test.rs

use doob::sync::adapters::BeadsAdapter;
use doob::sync::domain::IssueTracker;

#[test]
fn beads_adapter__returns_correct_name() {
    let adapter = BeadsAdapter::new();
    assert_eq!(adapter.name(), "beads");
}
