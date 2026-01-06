//! LLM集成模块（用于生成答案和LLM-as-a-Judge评估）

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// LLM配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// 提供商（openai、openai_compatible等）
    pub provider: String,
    /// API key（为空则使用本地退化逻辑）
    pub api_key: Option<String>,
    /// 模型名称
    pub model: String,
    /// 自定义基址（可用于兼容服务）
    pub base_url: Option<String>,
}

/// LLM客户端
pub struct LlmClient {
    config: LlmConfig,
    client: reqwest::Client,
}

impl LlmClient {
    /// 创建新的LLM客户端
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// 生成答案（基于检索到的记忆）
    pub async fn generate_answer(&self, question: &str, context: &[String]) -> Result<String> {
        // 无API Key时退化为直接拼接上下文，保证评测可离线运行
        if self.config.api_key.is_none() {
            return Ok(if !context.is_empty() {
                context.join(" ")
            } else {
                "I don't have enough information to answer this question.".to_string()
            });
        }

        // 目前实现 OpenAI / OpenAI 兼容接口
        match self.config.provider.as_str() {
            "openai" | "openai_compatible" => {
                let system_prompt = "You are a QA assistant. Answer the user question strictly based on the provided conversation context. If the answer is not contained, reply with \"I don't know.\" Keep answers concise.";
                let mut user_prompt = String::new();
                user_prompt.push_str("Context:\n");
                for (idx, snippet) in context.iter().enumerate() {
                    user_prompt.push_str(&format!("{}. {}\n", idx + 1, snippet));
                }
                user_prompt.push_str("\nQuestion: ");
                user_prompt.push_str(question);

                let messages = vec![
                    json!({"role": "system", "content": system_prompt}),
                    json!({"role": "user", "content": user_prompt}),
                ];

                self.call_openai_chat(messages).await
            }
            // 其他provider可按需扩展
            _ => Ok(context.first().cloned().unwrap_or_else(|| {
                "I don't have enough information to answer this question.".to_string()
            })),
        }
    }

    /// LLM-as-a-Judge评估
    pub async fn judge_answer(
        &self,
        question: &str,
        expected_answer: &str,
        actual_answer: &str,
    ) -> Result<f64> {
        // 无API Key时返回中性分数
        if self.config.api_key.is_none() {
            return Ok(0.5);
        }

        match self.config.provider.as_str() {
            "openai" | "openai_compatible" => {
                let system_prompt = "You are an evaluator. Give a numeric score between 0 and 1 (inclusive) representing how well the candidate answer matches the expected answer for the question. Consider factual correctness and completeness. Respond with only the number.";
                let user_prompt = format!(
                    "Question: {}\nExpected Answer: {}\nCandidate Answer: {}",
                    question, expected_answer, actual_answer
                );

                let messages = vec![
                    json!({"role": "system", "content": system_prompt}),
                    json!({"role": "user", "content": user_prompt}),
                ];

                let raw = self.call_openai_chat(messages).await?;
                Ok(Self::parse_score(&raw))
            }
            _ => Ok(0.5),
        }
    }

    async fn call_openai_chat(&self, messages: Vec<serde_json::Value>) -> Result<String> {
        let api_key = self
            .config
            .api_key
            .clone()
            .ok_or_else(|| anyhow::anyhow!("missing api_key for LLM call"))?;

        let base = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
        let url = format!("{}/chat/completions", base.trim_end_matches('/'));

        let body = json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": 0.2,
        });

        let resp = self
            .client
            .post(url)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let value: serde_json::Value = resp.json().await?;
        if !status.is_success() {
            bail!("LLM call failed ({}): {}", status, value);
        }

        let answer = value["choices"]
            .get(0)
            .and_then(|c| c["message"]["content"].as_str())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        if answer.is_empty() {
            bail!("LLM returned empty response");
        }

        Ok(answer)
    }

    fn parse_score(text: &str) -> f64 {
        for token in text.split_whitespace() {
            let cleaned = token.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
            if let Ok(v) = cleaned.parse::<f64>() {
                return v.clamp(0.0, 1.0);
            }
        }
        0.5
    }
}
