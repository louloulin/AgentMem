//! Environment Configuration Demo
//!
//! 演示如何使用环境变量配置 AgentMem 的各个组件。
//!
//! 本示例展示：
//! 1. 数据库配置（DATABASE_URL, AGENTMEM_DB_PATH, etc.）
//! 2. LLM 配置（OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.）
//! 3. 嵌入模型配置（AGENTMEM_EMBEDDING_PROVIDER, etc.）
//! 4. 向量存储配置（AGENTMEM_VECTOR_STORE, etc.）

use agent_mem_core::config_env::{
    get_embedding_config_from_env, get_llm_config_from_env, get_storage_config_from_env,
    get_vector_store_config_from_env, has_database_config, has_embedding_config, has_llm_config,
    has_vector_store_config,
};
use tracing::{info, warn};

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("env_config_demo=info")
        .init();

    info!("🔧 AgentMem 环境变量配置演示");
    info!("============================================================");
    info!("");

    // 1. 数据库配置
    demo_database_config();

    info!("");

    // 2. LLM 配置
    demo_llm_config();

    info!("");

    // 3. 嵌入模型配置
    demo_embedding_config();

    info!("");

    // 4. 向量存储配置
    demo_vector_store_config();

    info!("");
    info!("============================================================");
    info!("✅ 演示完成！");
    info!("");
    print_usage_examples();
}

fn demo_database_config() {
    info!("1️⃣  数据库配置");
    info!("------------------------------------------------------------");

    if has_database_config() {
        match get_storage_config_from_env() {
            Ok(config) => {
                info!("✅ 数据库配置已设置");
                info!("   后端: {:?}", config.backend);
                info!("   连接: {}", config.connection);
            }
            Err(e) => {
                warn!("❌ 数据库配置错误: {}", e);
            }
        }
    } else {
        info!("⚠️  未设置数据库配置，将使用默认值");
        match get_storage_config_from_env() {
            Ok(config) => {
                info!("   默认后端: {:?}", config.backend);
                info!("   默认连接: {}", config.connection);
            }
            Err(e) => {
                warn!("❌ 获取默认配置失败: {}", e);
            }
        }
    }

    info!("");
    info!("📝 支持的环境变量：");
    info!("   - DATABASE_URL: 完整的数据库连接字符串");
    info!("   - AGENTMEM_DB_PATH: LibSQL 数据库文件路径");
    info!("   - AGENTMEM_DB_BACKEND: 后端类型（postgres 或 libsql）");
}

fn demo_llm_config() {
    info!("2️⃣  LLM 配置");
    info!("------------------------------------------------------------");

    if has_llm_config() {
        let config = get_llm_config_from_env();
        info!("✅ LLM 配置已设置");
        if let Some(provider) = &config.provider {
            info!("   提供商: {}", provider);
        }
        if let Some(model) = &config.model {
            info!("   模型: {}", model);
        }
        if config.api_key.is_some() {
            info!("   API Key: ✅ 已设置");
        } else {
            info!("   API Key: ⚠️  未设置");
        }
    } else {
        info!("⚠️  未设置 LLM 配置");
        info!("   智能功能将不可用");
    }

    info!("");
    info!("📝 支持的环境变量：");
    info!("   - OPENAI_API_KEY: OpenAI API 密钥");
    info!("   - ANTHROPIC_API_KEY: Anthropic API 密钥");
    info!("   - AGENTMEM_LLM_PROVIDER: LLM 提供商（openai, anthropic, ollama）");
    info!("   - AGENTMEM_LLM_MODEL: LLM 模型名称");
}

fn demo_embedding_config() {
    info!("3️⃣  嵌入模型配置");
    info!("------------------------------------------------------------");

    if has_embedding_config() {
        let config = get_embedding_config_from_env();
        info!("✅ 嵌入模型配置已设置");
        if let Some(provider) = &config.provider {
            info!("   提供商: {}", provider);
        }
        if let Some(model) = &config.model {
            info!("   模型: {}", model);
        }
        if config.api_key.is_some() {
            info!("   API Key: ✅ 已设置");
        }
    } else {
        info!("⚠️  未设置嵌入模型配置");
        info!("   向量搜索功能将不可用");
    }

    info!("");
    info!("📝 支持的环境变量：");
    info!("   - AGENTMEM_EMBEDDING_PROVIDER: 嵌入提供商（openai, local）");
    info!("   - AGENTMEM_EMBEDDING_MODEL: 嵌入模型名称");
    info!("   - OPENAI_API_KEY: OpenAI API 密钥（如使用 OpenAI 嵌入）");
}

fn demo_vector_store_config() {
    info!("4️⃣  向量存储配置");
    info!("------------------------------------------------------------");

    if has_vector_store_config() {
        let config = get_vector_store_config_from_env();
        info!("✅ 向量存储配置已设置");
        if let Some(provider) = &config.provider {
            info!("   提供商: {}", provider);
        }
        if let Some(url) = &config.url {
            info!("   URL: {}", url);
        }
        if config.api_key.is_some() {
            info!("   API Key: ✅ 已设置");
        }
    } else {
        info!("⚠️  未设置向量存储配置");
        info!("   将使用默认的内存向量存储");
    }

    info!("");
    info!("📝 支持的环境变量：");
    info!("   - AGENTMEM_VECTOR_STORE: 向量存储提供商（qdrant, pinecone, weaviate）");
    info!("   - QDRANT_URL: Qdrant 服务器 URL");
    info!("   - PINECONE_API_KEY: Pinecone API 密钥");
    info!("   - WEAVIATE_URL: Weaviate 服务器 URL");
}

fn print_usage_examples() {
    info!("💡 使用示例：");
    info!("");
    info!("# 1. 最小配置（仅数据库）");
    info!("export AGENTMEM_DB_PATH=\"./data/memory.db\"");
    info!("cargo run --example env-config-demo");
    info!("");
    info!("# 2. 使用 OpenAI");
    info!("export OPENAI_API_KEY=\"sk-...\"");
    info!("export AGENTMEM_LLM_MODEL=\"gpt-4\"");
    info!("export AGENTMEM_EMBEDDING_MODEL=\"text-embedding-3-small\"");
    info!("cargo run --example env-config-demo");
    info!("");
    info!("# 3. 使用 PostgreSQL + Qdrant");
    info!("export DATABASE_URL=\"postgresql://user:pass@localhost/agentmem\"");
    info!("export AGENTMEM_VECTOR_STORE=\"qdrant\"");
    info!("export QDRANT_URL=\"http://localhost:6333\"");
    info!("cargo run --example env-config-demo");
    info!("");
    info!("# 4. 完整配置");
    info!("export DATABASE_URL=\"postgresql://user:pass@localhost/agentmem\"");
    info!("export OPENAI_API_KEY=\"sk-...\"");
    info!("export AGENTMEM_LLM_PROVIDER=\"openai\"");
    info!("export AGENTMEM_LLM_MODEL=\"gpt-4\"");
    info!("export AGENTMEM_EMBEDDING_PROVIDER=\"openai\"");
    info!("export AGENTMEM_EMBEDDING_MODEL=\"text-embedding-3-small\"");
    info!("export AGENTMEM_VECTOR_STORE=\"qdrant\"");
    info!("export QDRANT_URL=\"http://localhost:6333\"");
    info!("cargo run --example env-config-demo");
}

