# Sync Module Testing Guide

## Running Tests

### With Nextest (Recommended)

```bash
# Install nextest
cargo install cargo-nextest --locked

# Run all tests
cargo nextest run

# Run only unit tests
cargo nextest run --lib

# Run with coverage
cargo llvm-cov nextest --all-features --workspace --lcov --output-path lcov.info
```

### With Cargo Test

```bash
# Run all tests
cargo test

# Run specific test module
cargo test sync_service

# Run with output
cargo test -- --nocapture
```

## Test Organization

```
tests/
├── sync_domain_test.rs        # Domain model tests (10 tests)
├── sync_service_test.rs       # SyncService tests (5 tests)
├── beads_adapter_test.rs      # BeadsAdapter unit tests (1 test)
└── beads_integration_test.rs  # Integration tests (2 tests, require bd CLI)
```

## Coverage Goals

- **Overall:** 80%+ test coverage
- **Domain layer:** 90%+ (critical business logic)
- **Adapters:** 70%+ (integration tests may be limited)
- **CLI commands:** 60%+ (harder to test, validated manually)

## Writing Tests

Follow TDD approach:

1. Write failing test
2. Run test to verify it fails
3. Implement minimal code to make test pass
4. Run test to verify it passes
5. Commit

Use descriptive test names:

```rust
#[test]
fn sync_service__creates_issue__when_todo_is_pending() {
    // Test implementation
}
```

## Integration Tests

Integration tests require the `integration-tests` feature flag:

```bash
# Run integration tests
cargo test --features integration-tests

# Run specific integration test
cargo test --features integration-tests beads_adapter__creates_real_issue
```

### BeadsAdapter Integration Tests

The BeadsAdapter integration tests interact with the real `bd` CLI:

- **Test 1:** Creates a real issue in beads (skipped if bd not available)
- **Test 2:** Checks bd CLI availability

**Note:** Integration tests may create real issues. Clean up manually:

```bash
# List recent issues
bd list

# Close test issue
bd close <issue-id>
```

## Test Coverage Report

```bash
# Generate HTML coverage report
cargo llvm-cov nextest --html

# Open report
open target/llvm-cov/html/index.html
```

## Current Test Suite

### Domain Tests (sync_domain_test.rs)

- `todo_status__serializes_to_string`
- `todo_status__deserializes_from_string`
- `todo_status__supports_equality`
- `syncable_todo__creates_with_required_fields`
- `syncable_todo__serializes_to_json`
- `sync_record__stores_external_sync_data`
- `sync_record__serializes_with_optional_url`
- `sync_error__formats_provider_unavailable`
- `sync_error__formats_external_api_error`
- `sync_error__is_debug_formattable`
- `issue_tracker__returns_provider_name`
- `issue_tracker__checks_availability`
- `issue_tracker__creates_issue_when_available`

### Service Tests (sync_service_test.rs)

- `sync_service__creates_issue__when_todo_is_pending`
- `sync_service__creates_issue__when_todo_is_in_progress`
- `sync_service__rejects__when_provider_unavailable`
- `sync_service__handles_multiple_todos`
- `sync_service__partial_failure_continues`

### Adapter Tests (beads_adapter_test.rs)

- `beads_adapter__returns_correct_name`

### Integration Tests (beads_integration_test.rs)

- `beads_adapter__creates_real_issue__when_bd_cli_available` (conditional)
- `beads_adapter__is_available__when_bd_not_installed` (conditional)

## CI/CD Configuration

The nextest configuration includes a CI profile:

```toml
[profile.ci]
retries = 0
failure-output = "immediate-final"
success-output = "final"
```

Run in CI mode:

```bash
cargo nextest run --profile ci
```

## Debugging Failed Tests

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run with full output
cargo test -- --nocapture

# Run specific test with logging
RUST_LOG=debug cargo test sync_service__creates_issue
```
