// src/sync/adapters/mod.rs

#[cfg(feature = "bd")]
pub mod beads;

#[cfg(feature = "bd")]
pub use beads::BeadsAdapter;
