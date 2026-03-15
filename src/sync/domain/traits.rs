// src/sync/domain/traits.rs
//
// # Ports (Trait Definitions) for Sync Adapters
//
// This module defines the **ports** (interfaces) that external adapters must
// implement to integrate with the sync system. It applies the **Interface
// Segregation Principle (ISP)** - instead of one monolithic trait, we provide
// small, focused traits that adapters compose based on their capabilities.
//
// ## Why ISP?
//
// Different issue trackers have different capabilities:
// - Beads (bd CLI): Create only
// - GitHub Issues: Create, Update, Delete
// - Linear: Create, Update, Delete, Bidirectional sync
//
// With ISP, adapters implement only what they support:
//
// ```rust
// // BeadsAdapter - minimal (create only)
// impl Provider for BeadsAdapter { }
// impl HealthCheck for BeadsAdapter { }
// impl IssueCreator for BeadsAdapter { }
// // Automatically gets MinimalIssueTracker
//
// // GitHubAdapter - standard (create, update, delete)
// impl Provider for GitHubAdapter { }
// impl HealthCheck for GitHubAdapter { }
// impl IssueCreator for GitHubAdapter { }
// impl IssueUpdater for GitHubAdapter { }
// impl IssueDeleter for GitHubAdapter { }
// // Automatically gets StandardIssueTracker
// ```
//
// ## Trait Hierarchy
//
// ```text
// Provider (identity)
//     +
// HealthCheck (availability)
//     +
// IssueCreator (create)
//     = MinimalIssueTracker (auto trait)
//     +
// IssueUpdater (update)
//     +
// IssueDeleter (delete)
//     = StandardIssueTracker (auto trait)
//     +
// ExternalIssueReader (read)
//     +
// BatchIssueCreator (batch)
//     = FullIssueTracker (auto trait)
// ```
//
// ## Auto Traits
//
// Blanket implementations automatically provide composite traits:
// - `MinimalIssueTracker` = Provider + HealthCheck + IssueCreator
// - `StandardIssueTracker` = MinimalIssueTracker + IssueUpdater + IssueDeleter
// - `FullIssueTracker` = StandardIssueTracker + ExternalIssueReader + BatchIssueCreator
//
// ## Backward Compatibility
//
// The deprecated `IssueTracker` trait is automatically implemented for any
// type that implements the three minimal traits. This allows existing code
// to continue working during migration.

use super::SyncError;
use crate::sync::domain::{SyncRecord, SyncableTodo};

// ============================================================================
// TRAIT 1: Provider Metadata
// ============================================================================

/// Provider identity and capability information.
///
/// All adapters must implement this trait to identify themselves
/// and declare their supported features.
pub trait Provider: Send + Sync {
    /// Provider name (e.g., "beads", "github", "linear")
    fn name(&self) -> &str;

    /// Adapter version for compatibility checking
    fn version(&self) -> &str;

    /// Supported capabilities for feature detection
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }
}

/// Capability flags for feature detection
#[derive(Debug, Clone, Default)]
pub struct ProviderCapabilities {
    pub supports_update: bool,
    pub supports_delete: bool,
    pub supports_bidirectional_sync: bool,
    pub supports_batch_operations: bool,
    pub supports_webhooks: bool,
}

// ============================================================================
// TRAIT 2: Health Checking
// ============================================================================

/// Health and availability checking.
///
/// Separated from operations to allow health checks without mutations.
pub trait HealthCheck: Send + Sync {
    /// Quick availability check (CLI installed, credentials present, etc.)
    fn is_available(&self) -> Result<bool, SyncError>;

    /// Detailed health status with diagnostics
    fn health_status(&self) -> ProviderHealth {
        ProviderHealth {
            available: self.is_available().unwrap_or(false),
            latency_ms: None,
            last_error: None,
            rate_limit_remaining: None,
        }
    }
}

/// Detailed health information
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub available: bool,
    pub latency_ms: Option<u64>,
    pub last_error: Option<String>,
    pub rate_limit_remaining: Option<u32>,
}

// ============================================================================
// TRAIT 3: Issue Creation (Write-Only)
// ============================================================================

/// Issue creation capability.
///
/// This is the minimal interface for syncing todos to external systems.
/// Adapters that only support creating issues (like bd CLI) implement this.
pub trait IssueCreator: Send + Sync {
    fn create_issue(&self, todo: &SyncableTodo) -> Result<SyncRecord, SyncError>;
}

// ============================================================================
// TRAIT 4: Issue Updates
// ============================================================================

/// Issue update capability.
///
/// Only implement this if the provider supports updating existing issues.
/// Requires `IssueCreator` because updates imply creation capability.
pub trait IssueUpdater: IssueCreator {
    fn update_issue(&self, external_id: &str, todo: &SyncableTodo) -> Result<(), SyncError>;
}

// ============================================================================
// TRAIT 5: Issue Deletion
// ============================================================================

/// Issue deletion capability.
///
/// Separated from updates because some providers allow delete but not update.
pub trait IssueDeleter: Send + Sync {
    fn delete_issue(&self, external_id: &str) -> Result<(), SyncError>;
}

// ============================================================================
// TRAIT 6: External Issue Reading (Bidirectional Sync)
// ============================================================================

/// Read operations from external system.
///
/// Implement this to support pulling changes from external systems back to doob.
pub trait ExternalIssueReader: Send + Sync {
    /// Get a single issue by external ID
    fn get_issue(&self, external_id: &str) -> Result<SyncableTodo, SyncError>;

    /// List issues modified since a timestamp
    fn list_issues(&self, since: Option<&str>) -> Result<Vec<SyncableTodo>, SyncError>;
}

// ============================================================================
// TRAIT 7: Batch Operations (Performance)
// ============================================================================

/// Batch issue creation for performance.
///
/// Implement this to support efficient bulk syncing.
/// Default implementation falls back to sequential creation.
pub trait BatchIssueCreator: Send + Sync {
    fn create_issues(&self, todos: &[SyncableTodo]) -> Vec<Result<SyncRecord, SyncError>>;
}

// Default implementation: Sequential creation
impl<T: IssueCreator> BatchIssueCreator for T {
    fn create_issues(&self, todos: &[SyncableTodo]) -> Vec<Result<SyncRecord, SyncError>> {
        todos.iter().map(|todo| self.create_issue(todo)).collect()
    }
}

// ============================================================================
// COMPOSED TRAITS (Convenience Aliases)
// ============================================================================

/// Minimal tracker: Only creation capability.
///
/// Use this for simple CLI-based adapters like BeadsAdapter.
pub trait MinimalIssueTracker: Provider + HealthCheck + IssueCreator {}

/// Standard tracker: Creation, updates, and deletion.
///
/// Use this for full-featured API adapters like GitHub or Linear.
pub trait StandardIssueTracker: Provider + HealthCheck + IssueCreator + IssueUpdater + IssueDeleter {}

/// Full bidirectional tracker: All operations including external reads.
///
/// Use this for adapters that support two-way sync (Jira, Azure DevOps).
pub trait FullIssueTracker: StandardIssueTracker + ExternalIssueReader {}

/// High-performance tracker with batch operations.
///
/// Use this for adapters that can optimize batch syncing.
pub trait BatchIssueTracker: MinimalIssueTracker + BatchIssueCreator {}

// Blanket implementations for composed traits
impl<T> MinimalIssueTracker for T where T: Provider + HealthCheck + IssueCreator {}
impl<T> StandardIssueTracker for T where T: Provider + HealthCheck + IssueCreator + IssueUpdater + IssueDeleter {}
impl<T> FullIssueTracker for T where T: StandardIssueTracker + ExternalIssueReader {}
impl<T> BatchIssueTracker for T where T: MinimalIssueTracker + BatchIssueCreator {}

// ============================================================================
// BACKWARD COMPATIBILITY
// ============================================================================

/// Legacy monolithic trait (deprecated).
///
/// This trait is deprecated in favor of the segregated traits above.
/// Existing code using `IssueTracker` will continue to work, but new
/// code should use the composed traits (`MinimalIssueTracker`, etc.).
#[deprecated(
    since = "0.2.0",
    note = "Use MinimalIssueTracker, StandardIssueTracker, or FullIssueTracker instead"
)]
pub trait IssueTracker: Provider + HealthCheck + IssueCreator {
    // Empty - just a composition of the new traits
}

// Blanket implementation for backward compatibility
#[allow(deprecated)]
impl<T> IssueTracker for T where T: Provider + HealthCheck + IssueCreator {}
