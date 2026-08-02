//! Verifies the tracers-task integration surface stays public — an
//! accidental `mod` (dropping `pub`) would break `doob`'s ability to
//! construct a `DoobCheckpointStore` without anyone noticing until runtime.

#[test]
fn tracers_adapter_and_store_are_public() {
    let _: fn(&doob_core::models::todo::TodoStatus) -> tracers_task::TaskStatus =
        doob_core::tracers_adapter::todo_status_to_task_status;
}
