//! Agent Factory - 从AgentMem Agent配置创建LumosAI Agent

use crate::memory_adapter::AgentMemBackend;
use agent_mem::Memory as AgentMemApi;
use agent_mem_core::storage::models::Agent;
use anyhow::{Context, Result};
use lumosai_core::agent::{Agent as LumosAgent, BasicAgent};
use lumosai_core::llm::{providers, LlmProvider};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct LumosAgentFactory {
    memory_api: Arc<AgentMemApi>,
}

impl LumosAgentFactory {
    pub fn new(memory_api: Arc<AgentMemApi>) -> Self {
        Self { memory_api }
    }

    /// 根据AgentMem Agent配置创建LumosAI Agent (返回BasicAgent以支持streaming)
    pub async fn create_chat_agent(
        &self,
        agent: &Agent,
        user_id: &str,
    ) -> anyhow::Result<BasicAgent> {
        let total_start = std::time::Instant::now();
        info!("🏭 [FACTORY] Agent creation started");
        info!("   Agent: {}, User: {}", agent.id, user_id);

        // 1. 解析LLM配置
        let step1_start = std::time::Instant::now();
        let llm_config = self.parse_llm_config(agent)?;
        let step1_duration = step1_start.elapsed();
        info!("   ⏱️  [STEP1] Parse LLM config: {:?}", step1_duration);
        debug!(
            "Parsed LLM config: provider={}, model={}",
            llm_config
                .get("provider")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown"),
            llm_config
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
        );

        // 2. 创建LLM Provider
        let step2_start = std::time::Instant::now();
        let llm_provider = self.create_llm_provider(&llm_config)?;
        let step2_duration = step2_start.elapsed();
        info!("   ⏱️  [STEP2] Create LLM provider: {:?}", step2_duration);

        // 3. 创建Memory Backend并配置
        let step3_start = std::time::Instant::now();
        info!("   🔄 [STEP3] Creating memory backend...");
        let memory_backend = self.create_memory_backend(agent, user_id).await?;
        let step3_duration = step3_start.elapsed();
        info!("   ⏱️  [STEP3] Create memory backend: {:?}", step3_duration);

        if step3_duration.as_millis() > 100 {
            warn!("   ⚠️  Memory backend creation took > 100ms");
        }

        // 4. 使用AgentBuilder构建LumosAI Agent - 真正集成Memory Backend
        let agent_name = agent
            .name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("assistant");

        let step4_start = std::time::Instant::now();
        info!("   🔄 [STEP4] Building BasicAgent...");
        use lumosai_core::agent::AgentBuilder;

        // 优化的系统提示词，强调记忆能力
        let default_system_prompt = r#"You are an intelligent AI assistant with advanced memory capabilities.

**Your Memory System:**
- You have access to a semantic memory system that automatically retrieves relevant past conversations
- When answering, you will see relevant historical context from previous interactions
- This memory is personalized to each user and helps you provide contextual, consistent responses

**How to Use Your Memory:**
1. **Reference Past Context**: If you see relevant information in the conversation history, acknowledge and build upon it
2. **Maintain Continuity**: Remember user preferences, facts they've shared, and ongoing topics
3. **Be Specific**: When referencing past conversations, be specific about what you recall
4. **Ask for Clarification**: If memory seems incomplete or unclear, ask the user to clarify

**Response Guidelines:**
- Provide helpful, accurate, and contextually relevant answers
- Use a friendly and professional tone
- If you see relevant memories, explicitly reference them (e.g., "Based on what you told me before...")
- If no relevant memories exist, respond naturally without mentioning the lack of context
- Be concise but comprehensive

**Example:**
User: "黄是谁？"
Good Response: "根据我的记忆，黄是一位工程师，专注于AI开发。"
Bad Response: "我不知道黄是谁。"

Remember: Your memory system provides you with semantically relevant context. Use it wisely to deliver personalized experiences."#;

        let mut lumos_agent = AgentBuilder::new()
            .name(agent_name)
            .instructions(
                &agent
                    .system
                    .clone()
                    .unwrap_or_else(|| default_system_prompt.to_string()),
            )
            .model(llm_provider)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build agent: {}", e))?;

        let step4_duration = step4_start.elapsed();
        info!("   ⏱️  [STEP4] Build BasicAgent: {:?}", step4_duration);

        // 5. 设置Memory Backend
        let step5_start = std::time::Instant::now();
        info!("   🔄 [STEP5] Attaching memory backend...");
        lumos_agent = lumos_agent
            .with_memory(memory_backend)
            .map_err(|e| anyhow::anyhow!("Failed to attach memory backend: {}", e))?;
        let step5_duration = step5_start.elapsed();
        info!("   ⏱️  [STEP5] Attach memory: {:?}", step5_duration);

        // 验证Memory是否被正确设置
        use lumosai_core::agent::Agent;
        if lumos_agent.has_own_memory() {
            info!("   ✅ Memory verified: attached");
        } else {
            warn!("   ⚠️  Memory verification failed");
        }

        let total_duration = total_start.elapsed();
        info!("✅ [FACTORY] Total agent creation: {:?}", total_duration);

        if total_duration.as_millis() > 100 {
            warn!("⚠️  [FACTORY] Agent creation took > 100ms, breakdown:");
            warn!("   STEP1 (parse):   {:?}", step1_duration);
            warn!("   STEP2 (provider): {:?}", step2_duration);
            warn!("   STEP3 (memory):   {:?}", step3_duration);
            warn!("   STEP4 (build):    {:?}", step4_duration);
            warn!("   STEP5 (attach):   {:?}", step5_duration);
        }

        Ok(lumos_agent)
    }

    /// 创建chat agent并包装为trait object (向后兼容)
    pub async fn create_chat_agent_arc(
        &self,
        agent: &Agent,
        user_id: &str,
    ) -> anyhow::Result<Arc<dyn LumosAgent>> {
        let basic_agent = self.create_chat_agent(agent, user_id).await?;
        Ok(Arc::new(basic_agent))
    }

    fn parse_llm_config(&self, agent: &Agent) -> anyhow::Result<Value> {
        let mut llm_config_value = agent
            .llm_config
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Agent LLM config not set"))?;

        // 如果配置中没有api_key，从环境变量读取
        if llm_config_value
            .get("api_key")
            .map(|v| v.is_null())
            .unwrap_or(true)
        {
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

    fn create_llm_provider(&self, config: &Value) -> anyhow::Result<Arc<dyn LlmProvider>> {
        let api_key = config["api_key"]
            .as_str()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "API key not configured for provider: {}",
                    config["provider"]
                )
            })?
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
            "maas" => Arc::new(providers::huawei_maas(api_key, Some(model))),
            _ => return Err(anyhow::anyhow!("Unsupported LLM provider: {}. Supported: zhipu, openai, anthropic, deepseek, qwen, gemini, cohere, maas", provider_name)),
        };

        Ok(provider)
    }

    async fn create_memory_backend(
        &self,
        agent: &Agent,
        user_id: &str,
    ) -> anyhow::Result<Arc<dyn lumosai_core::memory::Memory>> {
        // ✅ 使用agent-mem的Memory API（完整的顶层接口）
        let backend = Arc::new(AgentMemBackend::new(
            self.memory_api.clone(),
            agent.id.clone(),
            user_id.to_string(),
        ));

        Ok(backend as Arc<dyn lumosai_core::memory::Memory>)
    }
}
