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

        if has_all_dirs {
            let single_hop = self.load_category("single_hop").await?;
            let multi_hop = self.load_category("multi_hop").await?;
            let temporal = self.load_category("temporal").await?;
            let open_domain = self.load_category("open_domain").await?;
            let adversarial = self.load_category("adversarial").await?;

            return Ok(LocomoDatasets {
                single_hop,
                multi_hop,
                temporal,
                open_domain,
                adversarial,
            });
        }

        tracing::info!("分类目录缺失，尝试直接解析原始 LoCoMo 数据集...");
        self.load_from_raw().await
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

    /// 从原始 LoCoMo JSON 中构建分类数据集
    async fn load_from_raw(&self) -> Result<LocomoDatasets> {
        let base = Path::new(&self.base_path);
        let candidates = [
            base.join("locomo10.json"),
            base.join("msc_personas_all.json"),
        ];

        let raw_path = candidates
            .iter()
            .find(|p| p.exists())
            .ok_or_else(|| anyhow!("未找到原始 LoCoMo 数据文件: {:?}", candidates))?;

        let raw_content = fs::read_to_string(raw_path)
            .with_context(|| format!("读取原始数据失败: {}", raw_path.display()))?;
        let personas: Vec<Value> = serde_json::from_str(&raw_content)
            .with_context(|| format!("解析原始数据失败: {}", raw_path.display()))?;

        let mut buckets: HashMap<String, Vec<ConversationSession>> = HashMap::new();

        for (p_idx, persona) in personas.iter().enumerate() {
            let messages = self.extract_messages(persona)?;
            let questions = self.extract_questions(persona)?;

            for (q_idx, qa) in questions.into_iter().enumerate() {
                // 每个问题生成一个会话，复用对话消息，便于测试覆盖
                let session = ConversationSession {
                    session_id: format!("persona{}_q{}", p_idx, q_idx),
                    timestamp: "unknown".to_string(),
                    messages: messages.clone(),
                    questions: vec![qa.clone()],
                };
                buckets.entry(qa.category.clone()).or_default().push(session);
            }
        }

        Ok(LocomoDatasets {
            single_hop: buckets.remove("single_hop").unwrap_or_default(),
            multi_hop: buckets.remove("multi_hop").unwrap_or_default(),
            temporal: buckets.remove("temporal").unwrap_or_default(),
            open_domain: buckets.remove("open_domain").unwrap_or_default(),
            adversarial: buckets.remove("adversarial").unwrap_or_default(),
        })
    }

    /// 从原始 LoCoMo 结构中提取所有对话消息
    fn extract_messages(&self, persona: &Value) -> Result<Vec<Message>> {
        let mut messages = Vec::new();
        if let Value::Object(map) = persona {
            for (k, v) in map {
                if let Some(stripped) = k.strip_prefix("session_") {
                    // 跳过 summary/date_time 字段
                    if stripped.ends_with("_summary") || stripped.ends_with("_date_time") {
                        continue;
                    }
                    if let Value::Array(items) = v {
                        for item in items {
                            if let Value::Object(obj) = item {
                                let speaker = obj
                                    .get("speaker")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                let content = obj
                                    .get("text")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                if !content.is_empty() {
                                    messages.push(Message {
                                        role: speaker,
                                        content,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        if messages.is_empty() {
            return Err(anyhow!("原始数据缺少对话消息字段"));
        }
        Ok(messages)
    }

    /// 从原始 LoCoMo 结构中提取问题与答案，并映射到类别
    fn extract_questions(&self, persona: &Value) -> Result<Vec<QuestionAnswer>> {
        let mut questions = Vec::new();
        if let Some(qa_list) = persona.get("qa").and_then(|v| v.as_array()) {
            for (idx, qa) in qa_list.iter().enumerate() {
                let question = qa
                    .get("question")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let expected_answer = qa
                    .get("answer")
                    .or_else(|| qa.get("adversarial_answer"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                let category_int = qa.get("category").and_then(|c| c.as_u64()).unwrap_or(0);
                let category = Self::map_category(category_int);
                let session_refs = Self::extract_session_refs(qa.get("evidence"));

                questions.push(QuestionAnswer {
                    question_id: format!("q_{}", idx),
                    category,
                    question,
                    expected_answer,
                    session_references: session_refs,
                });
            }
        }

        if questions.is_empty() {
            return Err(anyhow!("原始数据缺少 qa 字段或内容为空"));
        }
        Ok(questions)
    }

    /// 将类别数字映射到内部类别名称
    fn map_category(category: u64) -> String {
        match category {
            0 => "single_hop",
            1 => "multi_hop",
            2 => "temporal",
            3 => "open_domain",
            4 => "adversarial",
            _ => "open_domain",
        }
        .to_string()
    }

    /// 从 evidence 中提取 session 引用，形如 "D3:12" -> "session_3"
    fn extract_session_refs(evidence: Option<&Value>) -> Vec<String> {
        let mut refs = Vec::new();
        if let Some(Value::Array(items)) = evidence {
            for item in items {
                if let Some(ev_str) = item.as_str() {
                    if let Some(stripped) = ev_str.strip_prefix('D') {
                        let id_part: String = stripped.chars().take_while(|c| c.is_numeric()).collect();
                        if !id_part.is_empty() {
                            refs.push(format!("session_{}", id_part));
                        }
                    }
                }
            }
        }
        if refs.is_empty() {
            refs.push("session_all".to_string());
        }
        refs
    }
}
