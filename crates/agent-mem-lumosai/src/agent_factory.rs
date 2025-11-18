//! Agent Factory - 从AgentMem Agent配置创建LumosAI Agent

use lumosai_core::agent::{AgentBuilder, Agent as LumosAgent};
use lumosai_core::llm::providers;
use lumosai_core::llm::LlmProvider;
use agent_mem_core::storage::models::Agent;
use agent_mem_core::storage::factory::Repositories;
use agent_mem_core::engine::{MemoryEngine, MemoryEngineConfig};
use agent_mem_traits::LLMConfig;
use crate::memory_adapter::AgentMemBackend;
use std::sync::Arc;
use tracing::{debug, info};

pub struct LumosAgentFactory {
    repositories: Arc<Repositories>,
}

impl LumosAgentFactory {
    pub fn new(repositories: Arc<Repositories>) -> Self {
        Self { repositories }
    }
    
    /// 根据AgentMem Agent配置创建LumosAI Agent
    pub async fn create_chat_agent(
        &self,
        agent: &Agent,
        user_id: &str,
    ) -> anyhow::Result<Arc<dyn LumosAgent>> {
        info!("Creating LumosAI agent for: {} (user: {})", agent.id, user_id);
        
        // 1. 解析LLM配置
        let llm_config = self.parse_llm_config(agent)?;
        debug!("Parsed LLM config: provider={}, model={}", llm_config.provider, llm_config.model);
        
        // 2. 创建LLM Provider
        let llm_provider = self.create_llm_provider(&llm_config)?;
        debug!("Created LLM provider: {}", llm_config.provider);
        
        // 3. 创建Memory Backend
        let _memory_backend = self.create_memory_backend(agent, user_id).await?;
        debug!("Created AgentMem backend");
        
        // 4. 使用AgentBuilder构建LumosAI Agent
        let agent_name = agent.name.as_ref().map(|s| s.as_str()).unwrap_or("assistant");
        let lumos_agent = AgentBuilder::new()
            .name(agent_name)
            .instructions(&agent.system.clone().unwrap_or_else(|| 
                "You are a helpful AI assistant".to_string()
            ))
            .model(llm_provider)
            .build()?;
        
        info!("✅ Successfully created LumosAI agent: {}", agent_name);
        Ok(Arc::new(lumos_agent))
    }
    
    fn parse_llm_config(&self, agent: &Agent) -> anyhow::Result<LLMConfig> {
        let llm_config_value = agent.llm_config.clone()
            .ok_or_else(|| anyhow::anyhow!("Agent LLM config not set"))?;
        
        let mut llm_config: LLMConfig = 
            serde_json::from_value(llm_config_value)?;
        
        // 从环境变量读取API key (如果配置中没有)
        if llm_config.api_key.is_none() {
            let env_var_name = format!("{}_API_KEY", llm_config.provider.to_uppercase());
            if let Ok(api_key) = std::env::var(&env_var_name) {
                debug!("Loaded API key from environment: {}", env_var_name);
                llm_config.api_key = Some(api_key);
            }
        }
        
        Ok(llm_config)
    }
    
    fn create_llm_provider(
        &self,
        config: &LLMConfig,
    ) -> anyhow::Result<Arc<dyn LlmProvider>> {
        let api_key = config.api_key.clone()
            .ok_or_else(|| anyhow::anyhow!("API key not configured for provider: {}", config.provider))?;
        
        let provider: Arc<dyn LlmProvider> = match config.provider.as_str() {
            "zhipu" => Arc::new(providers::zhipu(api_key, Some(config.model.clone()))),
            "openai" => Arc::new(providers::openai(api_key, Some(config.model.clone()))),
            "anthropic" => Arc::new(providers::anthropic(api_key, Some(config.model.clone()))),
            "deepseek" => Arc::new(providers::deepseek(api_key, Some(config.model.clone()))),
            "qwen" => Arc::new(providers::qwen(api_key, Some(config.model.clone()))),
            "gemini" => Arc::new(providers::gemini(api_key, config.model.clone())),
            "cohere" => Arc::new(providers::cohere(api_key, config.model.clone())),
            // "mistral" => Arc::new(providers::mistral(api_key, Some(config.model.clone()))),
            // "perplexity" => Arc::new(providers::perplexity(api_key, Some(config.model.clone()))),
            _ => return Err(anyhow::anyhow!("Unsupported LLM provider: {}. Supported: zhipu, openai, anthropic, deepseek, qwen, gemini, cohere", config.provider)),
        };
        
        Ok(provider)
    }
    
    async fn create_memory_backend(
        &self,
        agent: &Agent,
        user_id: &str,
    ) -> anyhow::Result<Arc<dyn lumosai_core::memory::Memory>> {
        // 创建MemoryEngine with LibSQL Repository
        let memory_engine = Arc::new(MemoryEngine::with_repository(
            MemoryEngineConfig::default(),
            self.repositories.memories.clone(),
        ));
        
        // 包装为AgentMemBackend
        let backend = Arc::new(AgentMemBackend::new(
            memory_engine,
            agent.id.clone(),
            user_id.to_string(),
        ));
        
        Ok(backend as Arc<dyn lumosai_core::memory::Memory>)
    }
}
