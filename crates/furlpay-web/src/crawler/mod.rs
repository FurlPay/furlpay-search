//! Recursive web exploration, link graph construction, and bounded crawling.

pub mod crawl_policy;
pub mod frontier;
pub mod link_extractor;
pub mod link_scoring;
pub mod scheduler;

pub use crawl_policy::CrawlPolicy;
pub use frontier::{CrawlFrontier, CrawlItem};
pub use link_extractor::{LinkExtractor, DiscoveredLink};
pub use link_scoring::{LinkScorer, LinkScoreWeights};
pub use scheduler::{CrawlScheduler, CrawlResult, CrawledPage, LinkEdge};
