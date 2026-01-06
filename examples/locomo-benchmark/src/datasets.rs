//! 数据集加载模块

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// 对话会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSession {
    pub session_id: String,
    pub timestamp: String,
    pub messages: Vec<Message>,
    pub questions: Vec<QuestionAnswer>,
}

/// 问答对
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnswer {
    pub question_id: String,
    pub category: String,
    pub question: String,
    pub expected_answer: String,
    pub session_references: Vec<String>,
}

/// 完整数据集
#[derive(Debug, Clone)]
pub struct LocomoDatasets {
    pub single_hop: Vec<ConversationSession>,
    pub multi_hop: Vec<ConversationSession>,
    pub temporal: Vec<ConversationSession>,
    pub open_domain: Vec<ConversationSession>,
    pub adversarial: Vec<ConversationSession>,
}

/// 数据集加载器
pub struct DatasetLoader {
    base_path: String,
}

impl DatasetLoader {
    /// 创建新的数据集加载器
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: base_path.to_string(),
        }
    }

    /// 加载所有数据集
    pub async fn load_all(&self) -> Result<LocomoDatasets> {
        let categories = [
            "single_hop",
            "multi_hop",
            "temporal",
            "open_domain",
            "adversarial",
        ];
        let has_all_dirs = categories
            .iter()
            .all(|c| Path::new(&self.base_path).join(c).exists());

        if !has_all_dirs {
            return Err(anyhow!(
                "未找到完整的分类数据目录，请先运行转换脚本：\n\
                 python scripts/convert_locomo.py --input LoCoMo/data/locomo10.json --output data"
            ));
        }

        let single_hop = self.load_category("single_hop").await?;
        let multi_hop = self.load_category("multi_hop").await?;
        let temporal = self.load_category("temporal").await?;
        let open_domain = self.load_category("open_domain").await?;
        let adversarial = self.load_category("adversarial").await?;

        Ok(LocomoDatasets {
            single_hop,
            multi_hop,
            temporal,
            open_domain,
            adversarial,
        })
    }

    /// 加载特定类别的数据集（要求目录和数据真实存在）
    async fn load_category(&self, category: &str) -> Result<Vec<ConversationSession>> {
        let category_path = Path::new(&self.base_path).join(category);

        if !category_path.exists() {
            return Err(anyhow!(
                "数据集目录不存在: {:?}。请先运行数据转换脚本生成分类数据。",
                category_path
            ));
        }

        let mut sessions = Vec::new();

        // 读取目录中的所有JSON文件
        let entries = fs::read_dir(&category_path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let session: ConversationSession = serde_json::from_str(&content)?;
                sessions.push(session);
            }
        }

        if sessions.is_empty() {
            return Err(anyhow!(
                "未找到数据集文件: {:?}。请确认已完成真实数据转换。",
                category_path
            ));
        }

        Ok(sessions)
    }

    #[allow(dead_code)]
    /// 保留以备未来自动转换使用
    async fn load_from_raw(&self) -> Result<LocomoDatasets> {
        Err(anyhow!(
            "raw 数据转换已禁用，请先运行 scripts/convert_locomo.py 生成分类数据集"
        ))
    }

    #[allow(dead_code)]
    fn extract_messages(&self, _persona: &Value) -> Result<Vec<Message>> {
        Err(anyhow!("extract_messages 仅用于 raw 转换，当前未启用"))
    }

    #[allow(dead_code)]
    fn extract_questions(&self, _persona: &Value) -> Result<Vec<QuestionAnswer>> {
        Err(anyhow!("extract_questions 仅用于 raw 转换，当前未启用"))
    }

    #[allow(dead_code)]
    fn map_category(_category: u64) -> String {
        "open_domain".to_string()
    }

    #[allow(dead_code)]
    fn extract_session_refs(_evidence: Option<&Value>) -> Vec<String> {
        vec!["session_all".to_string()]
    }
}
