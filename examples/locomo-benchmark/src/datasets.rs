//! 数据集加载模块

use anyhow::Result;
use serde::{Deserialize, Serialize};
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

    /// 加载特定类别的数据集
    async fn load_category(&self, category: &str) -> Result<Vec<ConversationSession>> {
        let category_path = Path::new(&self.base_path).join(category);

        // 如果目录不存在，创建示例数据
        if !category_path.exists() {
            tracing::warn!(
                "数据集目录不存在: {:?}, 创建示例数据",
                category_path
            );
            return Ok(self.create_sample_data(category));
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

        // 如果没有找到数据，创建示例数据
        if sessions.is_empty() {
            tracing::warn!(
                "未找到数据集文件，使用示例数据: {:?}",
                category_path
            );
            return Ok(self.create_sample_data(category));
        }

        Ok(sessions)
    }

    /// 创建示例数据（用于测试）
    fn create_sample_data(&self, category: &str) -> Vec<ConversationSession> {
        match category {
            "single_hop" => vec![ConversationSession {
                session_id: "session_1".to_string(),
                timestamp: "2025-01-01T10:00:00Z".to_string(),
                messages: vec![
                    Message {
                        role: "user".to_string(),
                        content: "I love pizza and Italian food.".to_string(),
                    },
                    Message {
                        role: "assistant".to_string(),
                        content: "That's great! Italian food is delicious.".to_string(),
                    },
                ],
                questions: vec![QuestionAnswer {
                    question_id: "q_001".to_string(),
                    category: "single_hop".to_string(),
                    question: "What do I love?".to_string(),
                    expected_answer: "pizza and Italian food".to_string(),
                    session_references: vec!["session_1".to_string()],
                }],
            }],
            "multi_hop" => vec![
                ConversationSession {
                    session_id: "session_1".to_string(),
                    timestamp: "2025-01-01T10:00:00Z".to_string(),
                    messages: vec![Message {
                        role: "user".to_string(),
                        content: "I work at Google.".to_string(),
                    }],
                    questions: vec![],
                },
                ConversationSession {
                    session_id: "session_2".to_string(),
                    timestamp: "2025-01-02T10:00:00Z".to_string(),
                    messages: vec![Message {
                        role: "user".to_string(),
                        content: "I'm a software engineer.".to_string(),
                    }],
                    questions: vec![QuestionAnswer {
                        question_id: "q_001".to_string(),
                        category: "multi_hop".to_string(),
                        question: "What is my job at Google?".to_string(),
                        expected_answer: "software engineer".to_string(),
                        session_references: vec!["session_1".to_string(), "session_2".to_string()],
                    }],
                },
            ],
            "temporal" => vec![
                ConversationSession {
                    session_id: "session_1".to_string(),
                    timestamp: "2025-01-01T10:00:00Z".to_string(),
                    messages: vec![Message {
                        role: "user".to_string(),
                        content: "I'm planning a trip to Japan.".to_string(),
                    }],
                    questions: vec![],
                },
                ConversationSession {
                    session_id: "session_2".to_string(),
                    timestamp: "2025-01-15T10:00:00Z".to_string(),
                    messages: vec![Message {
                        role: "user".to_string(),
                        content: "I just returned from Japan.".to_string(),
                    }],
                    questions: vec![QuestionAnswer {
                        question_id: "q_001".to_string(),
                        category: "temporal".to_string(),
                        question: "When did I return from Japan?".to_string(),
                        expected_answer: "January 15, 2025".to_string(),
                        session_references: vec!["session_2".to_string()],
                    }],
                },
            ],
            "open_domain" => vec![ConversationSession {
                session_id: "session_1".to_string(),
                timestamp: "2025-01-01T10:00:00Z".to_string(),
                messages: vec![Message {
                    role: "user".to_string(),
                    content: "I'm reading a book about quantum physics.".to_string(),
                }],
                questions: vec![QuestionAnswer {
                    question_id: "q_001".to_string(),
                    category: "open_domain".to_string(),
                    question: "What is quantum physics?".to_string(),
                    expected_answer: "A branch of physics that studies matter and energy at the quantum level".to_string(),
                    session_references: vec!["session_1".to_string()],
                }],
            }],
            "adversarial" => vec![ConversationSession {
                session_id: "session_1".to_string(),
                timestamp: "2025-01-01T10:00:00Z".to_string(),
                messages: vec![Message {
                    role: "user".to_string(),
                    content: "I love programming.".to_string(),
                }],
                questions: vec![QuestionAnswer {
                    question_id: "q_001".to_string(),
                    category: "adversarial".to_string(),
                    question: "What is my favorite programming language that I never mentioned?".to_string(),
                    expected_answer: "I cannot answer this question as it was never mentioned in our conversation.".to_string(),
                    session_references: vec!["session_1".to_string()],
                }],
            }],
            _ => vec![],
        }
    }
}
