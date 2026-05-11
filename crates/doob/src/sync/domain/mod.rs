// src/sync/domain/mod.rs
//
// # Sync Domain Layer
//
// This module contains the **domain layer** for syncing todos to external issue trackers,
// following hexagonal architecture (ports & adapters) and domain-driven design (DDD).
//
// ## Architecture
//
// ```text
// ┌─────────────────────────────────────────────────┐
// │           Domain Layer (this module)            │
// │  ┌──────────────────────────────────────────┐   │
// │  │ Domain Models (types.rs)                 │   │
// │  │  - SyncableTodo                          │   │
// │  │  - SyncRecord                            │   │
// │  │  - TodoStatus                            │   │
// │  │  - SyncError                             │   │
// │  └──────────────────────────────────────────┘   │
// │                    ↑                             │
// │  ┌─────────────────┴────────────────────────┐   │
// │  │ Domain Service (service.rs)              │   │
// │  │  - SyncService<T: MinimalIssueTracker>   │   │
// │  └──────────────────────────────────────────┘   │
// │                    ↑                             │
// │  ┌─────────────────┴────────────────────────┐   │
// │  │ Ports/Traits (traits.rs)                 │   │
// │  │  - Provider (identity)                   │   │
// │  │  - HealthCheck (availability)            │   │
// │  │  - IssueCreator (write operations)       │   │
// │  │  - IssueUpdater (optional)               │   │
// │  │  - IssueDeleter (optional)               │   │
// │  │  - MinimalIssueTracker (auto trait)      │   │
// │  └──────────────────────────────────────────┘   │
// └───────────────────┬─────────────────────────────┘
//                     │ Implements
//                     ↓
//          ┌──────────────────────┐
//          │ Adapter Layer        │
//          │  - BeadsAdapter      │
//          │  - GitHubAdapter*    │
//          │  - LinearAdapter*    │
//          └──────────────────────┘
//               (* = future)
// ```
//
// ## Design Principles
//
// ### 1. **Interface Segregation (ISP)**
//
// Instead of one monolithic `IssueTracker` trait, we provide small, focused traits:
// - `Provider` - identity and capabilities
// - `HealthCheck` - availability checking
// - `IssueCreator` - issue creation (minimal requirement)
// - `IssueUpdater` - optional update capability
// - `IssueDeleter` - optional delete capability
//
// Adapters implement only what they support. For example, `BeadsAdapter` only
// implements `Provider + HealthCheck + IssueCreator` because bd CLI doesn't
// support updates or deletes.
//
// ### 2. **Dependency Inversion**
//
// - Domain layer defines **ports** (traits) that adapters must implement
// - Domain layer does **not** depend on concrete adapters
// - `SyncService` is generic over `T: MinimalIssueTracker`
// - Adapters depend on domain, not vice versa
//
// ### 3. **Domain-Driven Design**
//
// - **Value Objects**: `SyncableTodo`, `SyncRecord`, `TodoStatus`
// - **Domain Service**: `SyncService` orchestrates sync operations
// - **Ports**: Traits define interfaces to external systems
// - **No Infrastructure**: Domain has zero dependencies on HTTP, CLI, DB, etc.
//
// ## Usage Example
//
// ```rust
// use doob::sync::domain::{SyncService, SyncableTodo, TodoStatus};
// use doob::sync::adapters::BeadsAdapter;
//
// // Create adapter
// let adapter = BeadsAdapter::new();
//
// // Create service (generic over any MinimalIssueTracker)
// let service = SyncService::new(adapter);
//
// // Prepare todo
// let todo = SyncableTodo {
//     id: "1".to_string(),
//     title: "Fix bug".to_string(),
//     status: TodoStatus::Pending,
//     priority: 2,
//     // ... other fields
// };
//
// // Sync to external tracker
// match service.sync_todo(&todo) {
//     Ok(record) => println!("Synced to {} (ID: {})",
//                           record.provider,
//                           record.external_id),
//     Err(e) => eprintln!("Sync failed: {}", e),
// }
// ```
//
// ## Module Organization
//
// - **`types.rs`** - Domain models and error types
// - **`traits.rs`** - Port definitions (ISP traits)
// - **`service.rs`** - `SyncService` domain service
// - **`mod.rs`** - Public API and re-exports
//
// ## Performance Optimizations
//
// - `SyncService::sync_todos()` checks availability **once** upfront, not N times
// - This avoids N subprocess spawns for CLI-based adapters like BeadsAdapter
// - For 100 todos, this saves 100-500ms of subprocess overhead
//
// ## Testing
//
// See `/tests/common/sync_mocks.rs` for shared test utilities:
// - `MockMinimalTracker` - configurable mock for testing
// - `make_test_todo()` - test data factory
//
// ## Future Enhancements
//
// - Batch operations (`BatchIssueCreator` trait)
// - Bidirectional sync (read operations)
// - Webhook support for real-time updates
// - Async/await for concurrent operations

pub mod service;
pub mod traits;
pub mod types;

// Re-export commonly used items
pub use traits::{
    BatchIssueCreator, ExternalIssueReader, HealthCheck, IssueCreator, IssueDeleter, IssueUpdater,
    Provider, ProviderCapabilities, ProviderHealth,
};

pub use traits::{BatchIssueTracker, FullIssueTracker, MinimalIssueTracker, StandardIssueTracker};

pub use types::{SyncError, SyncRecord, SyncableTodo, TodoStatus};

pub use service::SyncService;

// Backward compatibility: re-export deprecated IssueTracker
#[allow(deprecated)]
pub use traits::IssueTracker;
