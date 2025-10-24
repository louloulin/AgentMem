//! 提示词管理器

use crate::prompts::templates::PromptTemplates;
use agent_mem_traits::{AgentMemError, Message, MessageRole, Result};
use std::collections::HashMap;

/// 提示词管理器
pub struct PromptManager {
    templates: PromptTemplates,
}

impl PromptManager {
    /// 创建新的提示词管理器
    pub fn new() -> Self {
        Self {
            templates: PromptTemplates::new(),
        }
    }

    /// 使用自定义模板创建提示词管理器
    pub fn with_templates(templates: PromptTemplates) -> Self {
        Self { templates }
    }

    /// 构建记忆提取提示词
    pub fn build_memory_extraction_prompt(&self, conversation: &str) -> Result<Vec<Message>> {
        let mut variables = HashMap::new();
        variables.insert("conversation".to_string(), conversation.to_string());

        let prompt = self
            .templates
            .render_template("memory_extraction", &variables)
            .ok_or_else(|| {
                AgentMemError::llm_error("Failed to render memory extraction template")
            })?;

        Ok(vec![Message {
            role: MessageRole::User,
            content: prompt,
            timestamp: None,
        }])
    }

    /// 构建记忆摘要提示词
    pub fn build_memory_summarization_prompt(&self, memories: &str) -> Result<Vec<Message>> {
        let mut variables = HashMap::new();
        variables.insert("memories".to_string(), memories.to_string());

        let prompt = self
            .templates
            .render_template("memory_summarization", &variables)
            .ok_or_else(|| {
                AgentMemError::llm_error("Failed to render memory summarization template")
            })?;

        Ok(vec![Message {
            role: MessageRole::User,
            content: prompt,
            timestamp: None,
        }])
    }

    /// 构建记忆冲突检测提示词
    pub fn build_memory_conflict_detection_prompt(
        &self,
        memory_a: &str,
        memory_b: &str,
    ) -> Result<Vec<Message>> {
        let mut variables = HashMap::new();
        variables.insert("memory_a".to_string(), memory_a.to_string());
        variables.insert("memory_b".to_string(), memory_b.to_string());

        let prompt = self
            .templates
            .render_template("memory_conflict_detection", &variables)
            .ok_or_else(|| {
                AgentMemError::llm_error("Failed to render memory conflict detection template")
            })?;

        Ok(vec![Message {
            role: MessageRole::User,
            content: prompt,
            timestamp: None,
        }])
    }

    /// 构建记忆重要性评分提示词
    pub fn build_memory_importance_scoring_prompt(
        &self,
        memory_content: &str,
        user_context: &str,
    ) -> Result<Vec<Message>> {
        let mut variables = HashMap::new();
        variables.insert("memory_content".to_string(), memory_content.to_string());
        variables.insert("user_context".to_string(), user_context.to_string());

        let prompt = self
            .templates
            .render_template("memory_importance_scoring", &variables)
            .ok_or_else(|| {
                AgentMemError::llm_error("Failed to render memory importance scoring template")
            })?;

        Ok(vec![Message {
            role: MessageRole::User,
            content: prompt,
            timestamp: None,
        }])
    }

    /// 构建记忆查询增强提示词
    pub fn build_memory_query_enhancement_prompt(&self, user_query: &str) -> Result<Vec<Message>> {
        let mut variables = HashMap::new();
        variables.insert("user_query".to_string(), user_query.to_string());

        let prompt = self
            .templates
            .render_template("memory_query_enhancement", &variables)
            .ok_or_else(|| {
                AgentMemError::llm_error("Failed to render memory query enhancement template")
            })?;

        Ok(vec![Message {
            role: MessageRole::User,
            content: prompt,
            timestamp: None,
        }])
    }

    /// 构建自定义提示词
    pub fn build_custom_prompt(
        &self,
        template_name: &str,
        variables: HashMap<String, String>,
    ) -> Result<Vec<Message>> {
        let prompt = self
            .templates
            .render_template(template_name, &variables)
            .ok_or_else(|| {
                AgentMemError::llm_error(format!("Failed to render template: {template_name}"))
            })?;

        Ok(vec![Message {
            role: MessageRole::User,
            content: prompt,
            timestamp: None,
        }])
    }

    /// 构建带系统提示的消息
    pub fn build_system_prompt(&self, system_message: &str, user_message: &str) -> Vec<Message> {
        vec![
            Message {
                role: MessageRole::System,
                content: system_message.to_string(),
                timestamp: None,
            },
            Message {
                role: MessageRole::User,
                content: user_message.to_string(),
                timestamp: None,
            },
        ]
    }

    /// 构建对话式提示词
    pub fn build_conversation_prompt(&self, messages: Vec<(MessageRole, String)>) -> Vec<Message> {
        messages
            .into_iter()
            .map(|(role, content)| Message {
                role,
                content,
                timestamp: None,
            })
            .collect()
    }

    /// 添加自定义模板
    pub fn add_template(&mut self, name: String, template: String) {
        self.templates.add_template(name, template);
    }

    /// 获取所有可用的模板名称
    pub fn get_available_templates(&self) -> Vec<&String> {
        self.templates.get_template_names()
    }

    /// 验证模板变量
    pub fn validate_template_variables(
        &self,
        template_name: &str,
        variables: &HashMap<String, String>,
    ) -> Result<()> {
        let template = self.templates.get_template(template_name).ok_or_else(|| {
            AgentMemError::llm_error(format!("Template not found: {template_name}"))
        })?;

        // 简单的变量验证：检查模板中的占位符是否都有对应的变量
        let mut missing_variables = Vec::new();

        // 查找所有 {variable} 格式的占位符，但忽略JSON格式的花括号
        let mut chars = template.chars().peekable();
        let mut in_json_block = false;

        while let Some(ch) = chars.next() {
            // 检测JSON代码块
            if ch == '`' {
                // 检查是否是```json开始
                let mut backticks = 1;
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '`' {
                        chars.next();
                        backticks += 1;
                    } else {
                        break;
                    }
                }

                if backticks >= 3 {
                    // 检查是否是json块
                    let mut next_chars = String::new();
                    for _ in 0..4 {
                        if let Some(&_next_ch) = chars.peek() {
                            next_chars.push(chars.next().unwrap());
                        }
                    }
                    if next_chars.starts_with("json") {
                        in_json_block = true;
                        continue;
                    }
                }
            }

            // 如果遇到结束的```，退出JSON块
            if in_json_block && ch == '`' {
                let mut backticks = 1;
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '`' {
                        chars.next();
                        backticks += 1;
                    } else {
                        break;
                    }
                }
                if backticks >= 3 {
                    in_json_block = false;
                }
                continue;
            }

            // 只在非JSON块中查找变量占位符
            if !in_json_block && ch == '{' {
                let mut variable_name = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // 消费 '}'
                        break;
                    }
                    variable_name.push(chars.next().unwrap());
                }

                if !variable_name.is_empty() && !variables.contains_key(&variable_name) {
                    missing_variables.push(variable_name);
                }
            }
        }

        if !missing_variables.is_empty() {
            return Err(AgentMemError::llm_error(format!(
                "Missing variables for template {template_name}: {missing_variables:?}"
            )));
        }

        Ok(())
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_manager_creation() {
        let manager = PromptManager::new();
        assert!(!manager.get_available_templates().is_empty());
    }

    #[test]
    fn test_build_memory_extraction_prompt() {
        let manager = PromptManager::new();
        let result = manager.build_memory_extraction_prompt("Hello, I love tennis!");

        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].content.contains("Hello, I love tennis!"));
    }

    #[test]
    fn test_build_system_prompt() {
        let manager = PromptManager::new();
        let messages =
            manager.build_system_prompt("You are a helpful assistant", "Hello, how are you?");

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, MessageRole::System);
        assert_eq!(messages[1].role, MessageRole::User);
    }

    #[test]
    fn test_build_conversation_prompt() {
        let manager = PromptManager::new();
        let conversation = vec![
            (MessageRole::System, "You are helpful".to_string()),
            (MessageRole::User, "Hello".to_string()),
            (MessageRole::Assistant, "Hi there!".to_string()),
        ];

        let messages = manager.build_conversation_prompt(conversation);
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_validate_template_variables() {
        let manager = PromptManager::new();
        let mut variables = HashMap::new();
        variables.insert("conversation".to_string(), "test".to_string());

        let result = manager.validate_template_variables("memory_extraction", &variables);
        if let Err(ref e) = result {
            println!("Validation error: {e}");
        }
        assert!(result.is_ok());

        // 测试缺少变量的情况
        let empty_variables = HashMap::new();
        let result = manager.validate_template_variables("memory_extraction", &empty_variables);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_custom_template() {
        let mut manager = PromptManager::new();
        manager.add_template("test".to_string(), "Hello {name}!".to_string());

        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "World".to_string());

        let result = manager.build_custom_prompt("test", variables);
        assert!(result.is_ok());
        assert!(result.unwrap()[0].content.contains("Hello World!"));
    }
}
