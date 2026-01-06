//! 预定义的提示词模板

use std::collections::HashMap;

/// 记忆提取提示词模板
pub const MEMORY_EXTRACTION_PROMPT: &str = r#"
你是一个专业的记忆提取助手。请从以下对话中提取重要的记忆信息。

对话内容：
{conversation}

请提取以下类型的记忆：
1. 事实性信息（人名、地点、时间、事件等）
2. 偏好和兴趣
3. 重要的决定或计划
4. 情感状态和态度
5. 技能和能力

请以JSON格式返回提取的记忆，格式如下：
```json
{
  "memories": [
    {
      "content": "记忆内容",
      "type": "episodic|semantic|procedural|working",
      "importance": 0.8,
      "entities": ["实体1", "实体2"],
      "relations": [{"from": "实体1", "to": "实体2", "relation": "关系类型"}]
    }
  ]
}
```
"#;

/// 记忆摘要提示词模板
pub const MEMORY_SUMMARIZATION_PROMPT: &str = r#"
你是一个专业的记忆摘要助手。请对以下记忆进行智能摘要。

原始记忆：
{memories}

请生成一个简洁但完整的摘要，保留所有重要信息。摘要应该：
1. 保持原始信息的准确性
2. 去除冗余和重复
3. 突出重要的关键点
4. 保持逻辑连贯性

请直接返回摘要内容，不需要额外的格式。
"#;

/// 记忆冲突检测提示词模板
pub const MEMORY_CONFLICT_DETECTION_PROMPT: &str = r#"
你是一个专业的记忆冲突检测助手。请检查以下记忆之间是否存在冲突。

记忆A：{memory_a}
记忆B：{memory_b}

请分析这两个记忆是否存在以下类型的冲突：
1. 事实冲突（相同事件的不同描述）
2. 时间冲突（时间线不一致）
3. 逻辑冲突（逻辑上不能同时为真）
4. 偏好冲突（相互矛盾的偏好）

请以JSON格式返回分析结果：
```json
{
  "has_conflict": true/false,
  "conflict_type": "fact|time|logic|preference|none",
  "confidence": 0.9,
  "explanation": "冲突说明",
  "resolution_suggestion": "解决建议"
}
```
"#;

/// 记忆重要性评分提示词模板
pub const MEMORY_IMPORTANCE_SCORING_PROMPT: &str = r#"
你是一个专业的记忆重要性评估助手。请为以下记忆评估重要性分数。

记忆内容：{memory_content}
用户上下文：{user_context}

请根据以下标准评估重要性（0.0-1.0）：
1. 个人相关性（0.3权重）
2. 情感强度（0.2权重）
3. 实用价值（0.2权重）
4. 独特性（0.15权重）
5. 时效性（0.15权重）

请以JSON格式返回评估结果：
```json
{
  "importance_score": 0.85,
  "reasoning": {
    "personal_relevance": 0.9,
    "emotional_intensity": 0.7,
    "practical_value": 0.8,
    "uniqueness": 0.9,
    "timeliness": 0.8
  },
  "explanation": "评分说明"
}
```
"#;

/// 记忆查询增强提示词模板
pub const MEMORY_QUERY_ENHANCEMENT_PROMPT: &str = r#"
你是一个专业的查询增强助手。请将用户的自然语言查询转换为更精确的搜索查询。

用户查询：{user_query}

请生成以下增强查询：
1. 关键词提取
2. 同义词扩展
3. 相关概念
4. 时间范围（如果适用）
5. 实体识别

请以JSON格式返回增强结果：
```json
{
  "enhanced_query": "增强后的查询",
  "keywords": ["关键词1", "关键词2"],
  "synonyms": ["同义词1", "同义词2"],
  "related_concepts": ["相关概念1", "相关概念2"],
  "time_range": "时间范围",
  "entities": ["实体1", "实体2"],
  "query_intent": "查询意图"
}
```
"#;

/// 提示词模板管理器
pub struct PromptTemplates {
    templates: HashMap<String, String>,
}

impl PromptTemplates {
    /// 创建新的提示词模板管理器
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        templates.insert(
            "memory_extraction".to_string(),
            MEMORY_EXTRACTION_PROMPT.to_string(),
        );
        templates.insert(
            "memory_summarization".to_string(),
            MEMORY_SUMMARIZATION_PROMPT.to_string(),
        );
        templates.insert(
            "memory_conflict_detection".to_string(),
            MEMORY_CONFLICT_DETECTION_PROMPT.to_string(),
        );
        templates.insert(
            "memory_importance_scoring".to_string(),
            MEMORY_IMPORTANCE_SCORING_PROMPT.to_string(),
        );
        templates.insert(
            "memory_query_enhancement".to_string(),
            MEMORY_QUERY_ENHANCEMENT_PROMPT.to_string(),
        );

        Self { templates }
    }

    /// 获取提示词模板
    pub fn get_template(&self, name: &str) -> Option<&String> {
        self.templates.get(name)
    }

    /// 添加自定义提示词模板
    pub fn add_template(&mut self, name: String, template: String) {
        self.templates.insert(name, template);
    }

    /// 移除提示词模板
    pub fn remove_template(&mut self, name: &str) -> Option<String> {
        self.templates.remove(name)
    }

    /// 获取所有模板名称
    pub fn get_template_names(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }

    /// 渲染模板（简单的字符串替换）
    pub fn render_template(
        &self,
        name: &str,
        variables: &HashMap<String, String>,
    ) -> Option<String> {
        if let Some(template) = self.get_template(name) {
            let mut rendered = template.clone();

            for (key, value) in variables {
                let placeholder = format!("{{{key}}}");
                rendered = rendered.replace(&placeholder, value);
            }

            Some(rendered)
        } else {
            None
        }
    }
}

impl Default for PromptTemplates {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_templates_creation() {
        let templates = PromptTemplates::new();
        assert!(templates.get_template("memory_extraction").is_some());
        assert!(templates.get_template("memory_summarization").is_some());
        assert!(templates.get_template("nonexistent").is_none());
    }

    #[test]
    fn test_add_custom_template() {
        let mut templates = PromptTemplates::new();
        templates.add_template(
            "custom".to_string(),
            "Custom template: {variable}".to_string(),
        );

        assert!(templates.get_template("custom").is_some());
    }

    #[test]
    fn test_render_template() {
        let templates = PromptTemplates::new();
        let mut variables = HashMap::new();
        variables.insert(
            "conversation".to_string(),
            "Hello, how are you?".to_string(),
        );

        let rendered = templates.render_template("memory_extraction", &variables);
        assert!(rendered.is_some());
        assert!(rendered.unwrap().contains("Hello, how are you?"));
    }

    #[test]
    fn test_get_template_names() {
        let templates = PromptTemplates::new();
        let names = templates.get_template_names();
        assert!(names.len() >= 5);
        assert!(names.contains(&&"memory_extraction".to_string()));
    }
}
