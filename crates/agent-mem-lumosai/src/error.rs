use thiserror::Error;

#[derive(Error, Debug)]
pub enum LumosIntegrationError {
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("Agent error: {0}")]
    Agent(String),
    
    #[error("LLM error: {0}")]
    Llm(String),
}
