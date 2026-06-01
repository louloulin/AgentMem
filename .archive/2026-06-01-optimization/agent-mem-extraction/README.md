# agent-mem-extraction

Extraction pipeline framework for AgentMem file-centric memory system.

## Overview

This crate provides a flexible, multi-stage extraction pipeline that transforms resources into structured memory items. The pipeline follows a 7-stage workflow:

1. **ResourceIngestor** - Mount and validate resources
2. **MultimodalPreprocessor** - Preprocess text, images, audio, video
3. **ItemExtractor** - Extract memory items from resources
4. **DedupeMerger** - Remove duplicates and merge similar items
5. **AutoCategorizer** - Automatically categorize memory items
6. **IndexPersistor** - Persist items and update search indexes
7. **ResponseBuilder** - Build response with extracted items

## Features

- Flexible pipeline architecture with configurable stages
- Support for multiple media types (text, image, audio, video)
- Deduplication with Jaccard similarity
- Automatic categorization based on content type
- Multi-tenancy support (user_id + optional agent_id)
- Comprehensive error handling
- Performance metrics tracking

## Usage

```rust
use agent_mem_extraction::{
    ExtractionPipeline,
    PipelineConfig,
    ExtractionInput,
    stages::{ResourceIngestor, ItemExtractor, DedupeMerger, AutoCategorizer},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create pipeline with default config
    let config = PipelineConfig::default();
    let mut pipeline = ExtractionPipeline::new(config);

    // Add stages (in order)
    pipeline.add_stage(Box::new(ResourceIngestor::new())).await?;
    pipeline.add_stage(Box::new(ItemExtractor::new())).await?;
    pipeline.add_stage(Box::new(DedupeMerger::new())).await?;
    pipeline.add_stage(Box::new(AutoCategorizer::new())).await?;

    // Execute pipeline
    let input = ExtractionInput::from_uri("file:///path/to/document.md", "user-123");
    let output = pipeline.execute(input).await?;

    println!("Extracted {} items", output.items.len());
    println!("Categories: {:?}", output.categories);

    Ok(())
}
```

## Architecture

```
ExtractionPipeline
    ├── stages: Vec<Box<dyn ExtractionStage>>
    ├── execute(input) -> Output
    └── config: PipelineConfig

ExtractionStage (trait)
    ├── process(input, output, context) -> Result<Output>
    ├── name() -> &str
    └── priority() -> StagePriority
```

## Configuration

```rust
let config = PipelineConfig {
    execution_mode: ExecutionMode::Sequential,  // Sequential, Parallel, Conditional
    enable_caching: true,
    stage_timeout_secs: 60,
    max_retries: 3,
    verbose: false,
    dedup_threshold: 0.85,
    category_confidence_threshold: 0.7,
};
```

## License

MIT OR Apache-2.0
