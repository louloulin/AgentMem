//! Standard extraction stages

pub mod categorizer;
pub mod deduper;
pub mod extractor;
pub mod indexer;
pub mod ingestor;
pub mod preprocessor;
pub mod response;

// Re-export standard stages
pub use categorizer::AutoCategorizer;
pub use deduper::DedupeMerger;
pub use extractor::ItemExtractor;
pub use indexer::IndexPersistor;
pub use ingestor::ResourceIngestor;
pub use preprocessor::MultimodalPreprocessor;
pub use response::ResponseBuilder;
