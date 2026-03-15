// src/sync/mod.rs
//
// # Sync Module: Todo Synchronization to External Issue Trackers
//
// This module provides functionality to sync todos from doob to external
// issue tracking systems (Beads, GitHub Issues, Linear, etc.).
//
// ## Architecture: Hexagonal (Ports & Adapters)
//
// ```text
// Application
//     ↓
// Domain Layer (business logic)
//   - Models: SyncableTodo, SyncRecord, SyncError
//   - Service: SyncService
//   - Ports: Provider, HealthCheck, IssueCreator
//     ↓
// Adapter Layer (external systems)
//   - BeadsAdapter (bd CLI)
//   - GitHubAdapter (future)
//   - LinearAdapter (future)
// ```
//
// ## Quick Start
//
// ```rust
// use doob::sync::domain::{SyncService, SyncableTodo, TodoStatus};
// use doob::sync::adapters::BeadsAdapter;
//
// // 1. Create an adapter for your issue tracker
// let adapter = BeadsAdapter::new();
//
// // 2. Create the sync service
// let service = SyncService::new(adapter);
//
// // 3. Check if provider is available
// if !service.is_available()? {
//     eprintln!("Beads CLI (bd) is not installed");
//     return;
// }
//
// // 4. Sync a todo
// let todo = SyncableTodo { /* ... */ };
// let record = service.sync_todo(&todo)?;
//
// println!("Created issue: {}", record.external_id);
// ```
//
// ## Modules
//
// - **`domain`** - Domain layer (pure business logic)
//   - Models, services, ports (traits), errors
//   - No dependencies on external systems
//
// - **`adapters`** - Adapter layer (external system integrations)
//   - `BeadsAdapter` - Beads.fyi via bd CLI
//   - Future: GitHub, Linear, Jira, etc.
//
// ## Supported Providers
//
// | Provider | Adapter | Create | Update | Delete | Bidirectional |
// |----------|---------|--------|--------|--------|---------------|
// | Beads    | `BeadsAdapter` | ✅ | ❌ | ❌ | ❌ |
// | GitHub   | *planned* | - | - | - | - |
// | Linear   | *planned* | - | - | - | - |
//
// ## Design Principles
//
// 1. **Interface Segregation** - Small, focused traits instead of monolithic interface
// 2. **Dependency Inversion** - Domain defines ports, adapters implement them
// 3. **Open/Closed** - Add new providers without modifying existing code
// 4. **Single Responsibility** - Each trait has one reason to change
//
// ## Documentation
//
// - See `domain/mod.rs` for detailed architecture documentation
// - See `/docs/sync-architecture-analysis.md` for coupling analysis
// - See `/tests/common/sync_mocks.rs` for testing utilities

pub mod domain;
pub mod adapters;
