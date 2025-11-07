//! Template Engine - 类似 Jinja2 的模板渲染引擎
//!
//! 支持：
//! - 变量替换: {{variable}}
//! - 条件语句: {% if condition %}...{% endif %}
//! - 循环语句: {% for item in list %}...{% endfor %}
//! - 过滤器: {{variable|filter}}

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 模板上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    variables: HashMap<String, String>,
    lists: HashMap<String, Vec<String>>,
}

impl TemplateContext {
    /// 创建新的模板上下文
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            lists: HashMap::new(),
        }
    }

    /// 设置变量
    pub fn set(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    /// 设置列表
    pub fn set_list(&mut self, key: String, values: Vec<String>) {
        self.lists.insert(key, values);
    }

    /// 获取变量
    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// 获取列表
    pub fn get_list(&self, key: &str) -> Option<&Vec<String>> {
        self.lists.get(key)
    }
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 模板引擎
pub struct TemplateEngine {
    /// 是否启用严格模式（未定义变量会报错）
    strict_mode: bool,
}

impl TemplateEngine {
    /// 创建新的模板引擎
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    /// 启用严格模式
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// 渲染模板
    pub fn render(&self, template: &str, context: &TemplateContext) -> Result<String> {
        let mut result = template.to_string();

        // 1. 处理条件语句
        result = self.process_conditionals(&result, context)?;

        // 2. 处理循环语句
        result = self.process_loops(&result, context)?;

        // 3. 处理变量替换
        result = self.process_variables(&result, context)?;

        Ok(result)
    }

    /// 处理变量替换 {{variable}}
    fn process_variables(&self, template: &str, context: &TemplateContext) -> Result<String> {
        let mut result = template.to_string();
        let var_pattern = regex::Regex::new(r"\{\{([^}]+)\}\}")
            .map_err(|e| AgentMemError::internal_error(format!("Failed to compile regex: {e}")))?;

        for cap in var_pattern.captures_iter(template) {
            let full_match = &cap[0];
            let var_expr = cap[1].trim();

            // 检查是否有过滤器
            let (var_name, filter) = if var_expr.contains('|') {
                let parts: Vec<&str> = var_expr.split('|').collect();
                (parts[0].trim(), Some(parts[1].trim()))
            } else {
                (var_expr, None)
            };

            // 获取变量值
            let value = if let Some(val) = context.get(var_name) {
                val.clone()
            } else if self.strict_mode {
                return Err(AgentMemError::validation_error(format!(
                    "Undefined variable: {var_name}"
                )));
            } else {
                String::new()
            };

            // 应用过滤器
            let filtered_value = if let Some(filter_name) = filter {
                self.apply_filter(&value, filter_name)?
            } else {
                value
            };

            result = result.replace(full_match, &filtered_value);
        }

        Ok(result)
    }

    /// 处理条件语句 {% if condition %}...{% endif %}
    fn process_conditionals(&self, template: &str, context: &TemplateContext) -> Result<String> {
        let mut result = template.to_string();
        let if_pattern = regex::Regex::new(r"(?s)\{%\s*if\s+([^%]+)\s*%\}(.*?)\{%\s*endif\s*%\}")
            .map_err(|e| {
            AgentMemError::internal_error(format!("Failed to compile regex: {e}"))
        })?;

        for cap in if_pattern.captures_iter(template) {
            let full_match = &cap[0];
            let condition = cap[1].trim();
            let content = &cap[2];

            // 评估条件
            let should_include = self.evaluate_condition(condition, context)?;

            let replacement = if should_include {
                content.to_string()
            } else {
                String::new()
            };

            result = result.replace(full_match, &replacement);
        }

        Ok(result)
    }

    /// 处理循环语句 {% for item in list %}...{% endfor %}
    fn process_loops(&self, template: &str, context: &TemplateContext) -> Result<String> {
        let mut result = template.to_string();
        let for_pattern = regex::Regex::new(
            r"(?s)\{%\s*for\s+(\w+)\s+in\s+(\w+)\s*%\}(.*?)\{%\s*endfor\s*%\}",
        )
        .map_err(|e| AgentMemError::internal_error(format!("Failed to compile regex: {e}")))?;

        for cap in for_pattern.captures_iter(template) {
            let full_match = &cap[0];
            let item_var = &cap[1];
            let list_name = &cap[2];
            let loop_body = &cap[3];

            // 获取列表
            let list = if let Some(list) = context.get_list(list_name) {
                list
            } else if self.strict_mode {
                return Err(AgentMemError::validation_error(format!(
                    "Undefined list: {list_name}"
                )));
            } else {
                &Vec::new()
            };

            // 渲染循环体
            let mut loop_result = String::new();
            for item in list {
                let mut item_context = context.clone();
                item_context.set(item_var.to_string(), item.clone());

                // 递归渲染循环体
                let rendered_body = self.process_variables(loop_body, &item_context)?;
                loop_result.push_str(&rendered_body);
            }

            result = result.replace(full_match, &loop_result);
        }

        Ok(result)
    }

    /// 评估条件表达式
    fn evaluate_condition(&self, condition: &str, context: &TemplateContext) -> Result<bool> {
        // 简单的条件评估：检查变量或列表是否存在且非空
        let var_name = condition.trim();

        // 先检查是否是变量
        if let Some(value) = context.get(var_name) {
            Ok(!value.is_empty())
        } else if let Some(list) = context.get_list(var_name) {
            // 检查是否是列表
            Ok(!list.is_empty())
        } else {
            Ok(false)
        }
    }

    /// 应用过滤器
    fn apply_filter(&self, value: &str, filter: &str) -> Result<String> {
        match filter {
            "upper" => Ok(value.to_uppercase()),
            "lower" => Ok(value.to_lowercase()),
            "trim" => Ok(value.trim().to_string()),
            "length" => Ok(value.len().to_string()),
            "capitalize" => {
                let mut chars = value.chars();
                match chars.next() {
                    None => Ok(String::new()),
                    Some(first) => Ok(first.to_uppercase().collect::<String>() + chars.as_str()),
                }
            }
            _ => Err(AgentMemError::validation_error(format!(
                "Unknown filter: {filter}"
            ))),
        }
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_substitution() {
        let engine = TemplateEngine::new();
        let mut context = TemplateContext::new();
        context.set("name".to_string(), "Alice".to_string());
        context.set("age".to_string(), "30".to_string());

        let template = "Hello {{name}}, you are {{age}} years old.";
        let result = engine.render(template, &context).unwrap();

        assert_eq!(result, "Hello Alice, you are 30 years old.");
    }

    #[test]
    fn test_filter_upper() {
        let engine = TemplateEngine::new();
        let mut context = TemplateContext::new();
        context.set("name".to_string(), "alice".to_string());

        let template = "Hello {{name|upper}}!";
        let result = engine.render(template, &context).unwrap();

        assert_eq!(result, "Hello ALICE!");
    }

    #[test]
    fn test_conditional() {
        let engine = TemplateEngine::new();
        let mut context = TemplateContext::new();
        context.set("show_greeting".to_string(), "yes".to_string());

        let template = "{% if show_greeting %}Hello!{% endif %}";
        let result = engine.render(template, &context).unwrap();

        assert_eq!(result, "Hello!");
    }

    #[test]
    fn test_loop() {
        let engine = TemplateEngine::new();
        let mut context = TemplateContext::new();
        context.set_list(
            "items".to_string(),
            vec![
                "apple".to_string(),
                "banana".to_string(),
                "cherry".to_string(),
            ],
        );

        let template = "{% for item in items %}{{item}}, {% endfor %}";
        let result = engine.render(template, &context).unwrap();

        assert_eq!(result, "apple, banana, cherry, ");
    }

    #[test]
    fn test_strict_mode() {
        let engine = TemplateEngine::new().with_strict_mode(true);
        let context = TemplateContext::new();

        let template = "Hello {{undefined_var}}!";
        let result = engine.render(template, &context);

        assert!(result.is_err());
    }
}
