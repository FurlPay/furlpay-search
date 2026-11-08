//! Search discovery, query planning, provider management, and deduplication.

pub mod query_planner;
pub mod deduplication;
pub mod provider_registry;

pub use query_planner::{QueryPlanner, SearchPlan, PlanMode, DiscoveryQuery, RecognizedEntity, EntityType};
pub use deduplication::Deduplicator;
pub use provider_registry::{ProviderRegistry, ProviderHealth};
