# Sync Module: Coupling Analysis

## Analysis Date: 2026-03-15

## Question: Is the "tight coupling between SyncService and domain models" a problem?

### Answer: **NO** - This is intentional and correct for hexagonal architecture.

## Context

The devloop analysis flagged:
> "Potential tight coupling between SyncService and domain models"

This raised concern about whether our architecture violates separation of concerns.

## Analysis

### 1. What Coupling Exists?

**Domain Layer** (`src/sync/domain/`):
- `SyncService<T>` depends on: `SyncableTodo`, `SyncRecord`, `TodoStatus`, `SyncError`
- Traits (`IssueCreator`, etc.) depend on: `SyncableTodo`, `SyncRecord`

**Dependency Graph**:
```
SyncService (domain service)
    ↓
MinimalIssueTracker (port/trait)
    ↓
SyncableTodo, SyncRecord (domain models)
```

### 2. Is This Coupling Appropriate?

**YES**, for the following reasons:

#### a) **Hexagonal Architecture Permits Domain Coupling**

In hexagonal architecture (ports & adapters):
- **Domain layer** = business logic + domain models + ports (interfaces)
- **Adapters** = implementations of ports that connect to external systems

The domain layer is **allowed and expected** to:
- ✅ Have domain services that depend on domain models
- ✅ Define ports (traits) that use domain types
- ✅ Have internal coupling within the domain layer

The domain layer must **NOT**:
- ❌ Depend on specific adapter implementations
- ❌ Depend on external frameworks
- ❌ Leak domain types outside the module boundary

#### b) **Our Implementation Respects These Boundaries**

**Evidence of Good Boundaries**:
```bash
$ rg "use.*sync::" src/ --type rust | grep -v "src/sync/"
# No results - sync types do not leak outside the sync module
```

- ✅ `SyncService` is generic over `MinimalIssueTracker` (port), not concrete adapters
- ✅ No code outside `/src/sync/` imports sync domain types
- ✅ Adapters (`BeadsAdapter`) depend on domain, not vice versa
- ✅ Domain types are defined within the domain layer

#### c) **Domain-Driven Design Principle**

From DDD:
> "A domain service can depend on domain entities and value objects. This is not coupling - it's cohesion."

- `SyncableTodo` is a **value object** (data structure for sync operations)
- `SyncRecord` is a **value object** (sync metadata)
- `TodoStatus` is a **value object** (enum)
- `SyncService` is a **domain service** (orchestrates sync operations)

Domain services **should** know about their domain models.

### 3. What Would Be Problematic Coupling?

The following would be problems:

#### ❌ **Adapter Leakage**
```rust
// BAD: SyncService depends on concrete adapter
pub struct SyncService {
    tracker: BeadsAdapter,  // ← Violates dependency inversion
}
```

**Our Code**: ✅ Uses generic `T: MinimalIssueTracker`

#### ❌ **Domain Leakage**
```rust
// BAD: Main application code uses sync domain types
// src/main.rs
use doob::sync::domain::SyncableTodo;  // ← Domain type escapes boundary
```

**Our Code**: ✅ No external imports of sync domain types

#### ❌ **Infrastructure in Domain**
```rust
// BAD: Domain depends on external library
use reqwest::Client;

pub trait IssueCreator {
    fn create_issue(&self, client: &Client) -> Result<...>;  // ← Infrastructure leak
}
```

**Our Code**: ✅ Domain layer has no infrastructure dependencies

### 4. Actual Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│           Application Layer (future)            │
│  - CLI commands that orchestrate sync           │
└───────────────────┬─────────────────────────────┘
                    │ Uses (future)
                    ↓
┌─────────────────────────────────────────────────┐
│              Sync Domain Layer                  │
│  ┌──────────────────────────────────────────┐   │
│  │ Domain Models (value objects)            │   │
│  │  - SyncableTodo                          │   │
│  │  - SyncRecord                            │   │
│  │  - TodoStatus                            │   │
│  │  - SyncError                             │   │
│  └──────────────────────────────────────────┘   │
│                    ↑                             │
│  ┌─────────────────┴────────────────────────┐   │
│  │ Domain Service                           │   │
│  │  - SyncService<T: MinimalIssueTracker>   │   │
│  └──────────────────────────────────────────┘   │
│                    ↑                             │
│  ┌─────────────────┴────────────────────────┐   │
│  │ Ports (traits)                           │   │
│  │  - Provider                              │   │
│  │  - HealthCheck                           │   │
│  │  - IssueCreator                          │   │
│  │  - MinimalIssueTracker (auto trait)      │   │
│  └──────────────────────────────────────────┘   │
└───────────────────┬─────────────────────────────┘
                    │ Implements
                    ↓
┌─────────────────────────────────────────────────┐
│              Adapter Layer                      │
│  - BeadsAdapter (implements ports)              │
│  - Future: GitHubAdapter, LinearAdapter, etc.   │
└─────────────────────────────────────────────────┘
```

**Internal Coupling** (✅ OK):
- SyncService → domain models
- Ports → domain models

**External Coupling** (❌ None):
- No external code → domain models
- No domain → adapters
- No domain → infrastructure

## Conclusion

The coupling between `SyncService` and domain models (`SyncableTodo`, `SyncRecord`) is:

1. **Intentional** - part of domain-driven design
2. **Appropriate** - follows hexagonal architecture principles
3. **Contained** - does not leak outside module boundaries
4. **Correct** - domain services should know about domain models

### Recommendation

**No action needed.** The devloop analysis is a false positive. The coupling it detected is:
- Within the domain layer (SyncService knows about SyncableTodo)
- Properly encapsulated (no leakage outside /src/sync/)
- Following DDD and hexagonal architecture best practices

This is **cohesion**, not problematic coupling.

## Future Monitoring

Watch for these actual coupling problems:
- ❌ Code outside `/src/sync/` importing `SyncableTodo`, `SyncRecord`, etc.
- ❌ Domain layer depending on specific adapters (BeadsAdapter, etc.)
- ❌ Domain layer depending on infrastructure (HTTP clients, CLI tools, etc.)
- ❌ Adapters depending on each other

Current status: **None of these problems exist** ✅
