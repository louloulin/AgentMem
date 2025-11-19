//! 自动配置模块
//!
//! 负责从环境变量自动检测和配置各种组件

use crate::orchestrator::OrchestratorConfig;
use agent_mem_traits::Result;
use std::env;
use tracing::{info, warn};

/// 自动配置检测器
pub struct AutoConfig;

impl AutoConfig {
    /// 检测环境并生成配置
    ///
    /// 检测顺序：
    /// 1. 检测 LLM 提供商 (OPENAI_API_KEY, DEEPSEEK_API_KEY, etc.)
    /// 2. 检测存储后端 (DATABASE_URL, AGENTMEM_DB_PATH)
    /// 3. 检测向量存储 (LANCEDB_PATH, QDRANT_URL)
    /// 4. 决定是否启用智能功能
    pub async fn detect() -> Result<OrchestratorConfig> {
        info!("开始自动配置检测");

        let mut config = OrchestratorConfig::default();

        // 检测 LLM 提供商
        if let Some((provider, model)) = Self::detect_llm_provider() {
            info!("检测到 LLM 提供商: {} ({})", provider, model);
            config.llm_provider = Some(provider);
            config.llm_model = Some(model);
            config.enable_intelligent_features = true;
        } else {
            warn!("未检测到 LLM API Key，智能功能将被禁用");
            config.enable_intelligent_features = false;
        }

        // 检测 Embedder
        if let Some((provider, model)) = Self::detect_embedder() {
            info!("检测到 Embedder: {} ({})", provider, model);
            config.embedder_provider = Some(provider);
            config.embedder_model = Some(model);
        }

        // 检测存储后端
        if let Some(storage_url) = Self::detect_storage() {
            info!("检测到存储后端: {}", storage_url);
            config.storage_url = Some(storage_url);
        } else {
            // 默认使用 LibSQL
            let default_path = "agentmem.db";
            info!("使用默认存储: libsql://{}", default_path);
            config.storage_url = Some(format!("libsql://{}", default_path));
        }

        // 检测向量存储
        if let Some(vector_url) = Self::detect_vector_store() {
            info!("检测到向量存储: {}", vector_url);
            config.vector_store_url = Some(vector_url);
        }

        Ok(config)
    }

    /// 检测 LLM 提供商
    fn detect_llm_provider() -> Option<(String, String)> {
        // 检测智谱 AI (Zhipu)
        if env::var("ZHIPU_API_KEY").is_ok() {
            let model = env::var("ZHIPU_MODEL").unwrap_or_else(|_| "glm-4.6".to_string());
            return Some(("zhipu".to_string(), model));
        }

        // 检测 OpenAI
        if env::var("OPENAI_API_KEY").is_ok() {
            let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4".to_string());
            return Some(("openai".to_string(), model));
        }

        // 检测 Huawei MaaS
        if env::var("HUAWEI_MAAS_API_KEY").is_ok() {
            let model = env::var("HUAWEI_MAAS_MODEL").unwrap_or_else(|_| "deepseek-v3.2-exp".to_string());
            return Some(("huawei_maas".to_string(), model));
        }

        // 检测 DeepSeek
        if env::var("DEEPSEEK_API_KEY").is_ok() {
            let model = env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string());
            return Some(("deepseek".to_string(), model));
        }

        // 检测 Anthropic
        if env::var("ANTHROPIC_API_KEY").is_ok() {
            let model = env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());
            return Some(("anthropic".to_string(), model));
        }

        // 检测 Ollama (本地)
        if env::var("OLLAMA_HOST").is_ok() || env::var("OLLAMA_MODEL").is_ok() {
            let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2".to_string());
            return Some(("ollama".to_string(), model));
        }

        None
    }

    /// 检测 Embedder
    fn detect_embedder() -> Option<(String, String)> {
        // 检测 OpenAI Embedder
        if env::var("OPENAI_API_KEY").is_ok() {
            let model = env::var("OPENAI_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string());
            return Some(("openai".to_string(), model));
        }

        // 检测 Ollama Embedder
        if env::var("OLLAMA_HOST").is_ok() || env::var("OLLAMA_EMBEDDING_MODEL").is_ok() {
            let model = env::var("OLLAMA_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "nomic-embed-text".to_string());
            return Some(("ollama".to_string(), model));
        }

        None
    }

    /// 检测存储后端
    fn detect_storage() -> Option<String> {
        // 检测 PostgreSQL
        if let Ok(url) = env::var("DATABASE_URL") {
            if url.starts_with("postgres://") || url.starts_with("postgresql://") {
                return Some(url);
            }
        }

        // 检测 LibSQL
        if let Ok(path) = env::var("AGENTMEM_DB_PATH") {
            return Some(format!("libsql://{}", path));
        }

        None
    }

    /// 检测向量存储
    fn detect_vector_store() -> Option<String> {
        // 检测 LanceDB
        if let Ok(path) = env::var("LANCEDB_PATH") {
            return Some(format!("lancedb://{}", path));
        }

        // 检测 Qdrant
        if let Ok(url) = env::var("QDRANT_URL") {
            return Some(url);
        }

        // 检测 Pinecone
        if let Ok(api_key) = env::var("PINECONE_API_KEY") {
            return Some(format!("pinecone://{}", api_key));
        }

        None
    }
}
