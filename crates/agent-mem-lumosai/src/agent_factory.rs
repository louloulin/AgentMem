//! Agent Factory - 从AgentMem Agent配置创建LumosAI Agent

use anyhow::{Context, Result};
use agent_mem_core::engine::{MemoryEngine, MemoryEngineConfig};
use agent_mem_core::storage::models::Agent;
use agent_mem_core::storage::factory::Repositories;
use lumosai_core::agent::Agent as LumosAgent;
use lumosai_core::llm::{LlmProvider, providers};
use crate::memory_adapter::AgentMemBackend;
use std::sync::Arc;
use serde_json::Value;
use tracing::{debug, info, warn};

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
        debug!("Parsed LLM config: provider={}, model={}", 
            llm_config.get("provider").and_then(|v| v.as_str()).unwrap_or("unknown"),
            llm_config.get("model").and_then(|v| v.as_str()).unwrap_or("unknown"));
        
        // 2. 创建LLM Provider
        let llm_provider = self.create_llm_provider(&llm_config)?;
        debug!("Created LLM provider: {}", llm_config.get("provider").and_then(|v| v.as_str()).unwrap_or("unknown"));
        
        // 3. 创建Memory Backend并配置
        let memory_backend = self.create_memory_backend(agent, user_id).await?;
        debug!("Created AgentMem backend");
        
        // 4. 使用AgentBuilder构建LumosAI Agent - 真正集成Memory Backend
        let agent_name = agent.name.as_ref().map(|s| s.as_str()).unwrap_or("assistant");
        
        // ✅ 关键修复：使用AgentBuilder.build_async()而不是build()
        // build_async()支持异步操作，且能正确处理memory字段
        use lumosai_core::agent::AgentBuilder;
        
        let mut lumos_agent = AgentBuilder::new()
            .name(agent_name)
            .instructions(&agent.system.clone().unwrap_or_else(|| 
                "You are a helpful AI assistant".to_string()
            ))
            .model(llm_provider)
            .build()  // 先build基础Agent
            .map_err(|e| anyhow::anyhow!("Failed to build agent: {}", e))?;
        
        // ✅ 设置Memory Backend
        lumos_agent = lumos_agent.with_memory(memory_backend);
        
        // 验证Memory是否被正确设置
        use lumosai_core::agent::Agent;
        if lumos_agent.has_own_memory() {
            info!("✅ Memory Backend attached and verified!");
            if let Some(mem) = lumos_agent.get_memory() {
                info!("✅ get_memory() returns Some - Memory is accessible");
                drop(mem); // 释放引用
            } else {
                warn!("⚠️  has_own_memory()=true but get_memory()=None - This is a bug!");
            }
        } else {
            warn!("❌ Memory Backend NOT attached - has_own_memory()=false");
        }
        
        info!("✅ Successfully created LumosAI agent with integrated memory: {}", agent_name);
        
        Ok(Arc::new(lumos_agent))
    }
    
    fn parse_llm_config(&self, agent: &Agent) -> anyhow::Result<Value> {
        let mut llm_config_value = agent.llm_config.clone()
            .ok_or_else(|| anyhow::anyhow!("Agent LLM config not set"))?;
        
        // 如果配置中没有api_key，从环境变量读取
        if llm_config_value.get("api_key").map(|v| v.is_null()).unwrap_or(true) {
            if let Some(provider) = llm_config_value.get("provider").and_then(|v| v.as_str()) {
                let env_var_name = format!("{}_API_KEY", provider.to_uppercase());
                if let Ok(api_key) = std::env::var(&env_var_name) {
                    debug!("Loaded API key from environment: {}", env_var_name);
                    if let Some(obj) = llm_config_value.as_object_mut() {
                        obj.insert("api_key".to_string(), Value::String(api_key));
                    }
                }
            }
        }
        
        Ok(llm_config_value)
    }
    
    fn create_llm_provider(
        &self,
        config: &Value,
    ) -> anyhow::Result<Arc<dyn LlmProvider>> {
        let api_key = config["api_key"].as_str()
            .ok_or_else(|| anyhow::anyhow!("API key not configured for provider: {}", config["provider"]))?
            .to_string();
        let provider_name = config["provider"].as_str().unwrap();
        let model = config["model"].as_str().unwrap().to_string();
        
        let provider: Arc<dyn LlmProvider> = match provider_name {
            "zhipu" => Arc::new(providers::zhipu(api_key, Some(model))),
            "openai" => Arc::new(providers::openai(api_key, Some(model))),
            "anthropic" => Arc::new(providers::anthropic(api_key, Some(model))),
            "deepseek" => Arc::new(providers::deepseek(api_key, Some(model))),
            "qwen" => Arc::new(providers::qwen(api_key, Some(model))),
            "gemini" => Arc::new(providers::gemini(api_key, model)),
            "cohere" => Arc::new(providers::cohere(api_key, model)),
            _ => return Err(anyhow::anyhow!("Unsupported LLM provider: {}. Supported: zhipu, openai, anthropic, deepseek, qwen, gemini, cohere", provider_name)),
        };
        
        Ok(provider)
    }
    
    async fn create_memory_backend(
        &self,
        agent: &Agent,
        user_id: &str,
    ) -> anyhow::Result<Arc<dyn lumosai_core::memory::Memory>> {
        // ✅ 使用 Repositories（包含完整的 Memory API）
        let backend = Arc::new(AgentMemBackend::new(
            self.repositories.clone(),
            agent.id.clone(),
            user_id.to_string(),
        ));
        
        Ok(backend as Arc<dyn lumosai_core::memory::Memory>)
    }
}
