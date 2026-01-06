//! Tool schema definition and validation
//!
//! This module provides JSON Schema generation and validation for tool parameters,
//! inspired by MIRIX's schema_generator.py but with Rust's type safety.

use crate::error::{ToolError, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Tool schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Parameter schema
    pub parameters: ParameterSchema,
}

/// Parameter schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// Type (always "object" for tool parameters)
    #[serde(rename = "type")]
    pub param_type: String,
    /// Property definitions
    pub properties: HashMap<String, PropertySchema>,
    /// Required parameters
    pub required: Vec<String>,
}

/// Property schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    /// Property type (string, number, boolean, array, object)
    #[serde(rename = "type")]
    pub prop_type: String,
    /// Property description
    pub description: String,
    /// Enum values (for string types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    /// Default value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    /// Minimum value (for numbers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// Maximum value (for numbers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    /// Array item schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<PropertySchema>>,
}

impl ToolSchema {
    /// Create a new tool schema
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: ParameterSchema {
                param_type: "object".to_string(),
                properties: HashMap::new(),
                required: Vec::new(),
            },
        }
    }

    /// Add a parameter to the schema
    pub fn add_parameter(
        mut self,
        name: impl Into<String>,
        prop_schema: PropertySchema,
        required: bool,
    ) -> Self {
        let name = name.into();
        if required {
            self.parameters.required.push(name.clone());
        }
        self.parameters.properties.insert(name, prop_schema);
        self
    }

    /// Validate arguments against the schema
    pub fn validate(&self, args: &Value) -> ToolResult<()> {
        let obj = args
            .as_object()
            .ok_or_else(|| ToolError::ValidationFailed("Expected object".to_string()))?;

        // Check required parameters
        for required in &self.parameters.required {
            if !obj.contains_key(required) {
                return Err(ToolError::ValidationFailed(format!(
                    "Missing required parameter: {required}"
                )));
            }
        }

        // Validate each parameter
        for (key, value) in obj {
            if let Some(prop_schema) = self.parameters.properties.get(key) {
                prop_schema.validate(value)?;
            } else {
                return Err(ToolError::ValidationFailed(format!(
                    "Unknown parameter: {key}"
                )));
            }
        }

        Ok(())
    }

    /// Convert to JSON Schema format
    pub fn to_json_schema(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl PropertySchema {
    /// Create a string property
    pub fn string(description: impl Into<String>) -> Self {
        Self {
            prop_type: "string".to_string(),
            description: description.into(),
            enum_values: None,
            default: None,
            minimum: None,
            maximum: None,
            items: None,
        }
    }

    /// Create a number property
    pub fn number(description: impl Into<String>) -> Self {
        Self {
            prop_type: "number".to_string(),
            description: description.into(),
            enum_values: None,
            default: None,
            minimum: None,
            maximum: None,
            items: None,
        }
    }

    /// Create a boolean property
    pub fn boolean(description: impl Into<String>) -> Self {
        Self {
            prop_type: "boolean".to_string(),
            description: description.into(),
            enum_values: None,
            default: None,
            minimum: None,
            maximum: None,
            items: None,
        }
    }

    /// Create an array property
    pub fn array(description: impl Into<String>, items: PropertySchema) -> Self {
        Self {
            prop_type: "array".to_string(),
            description: description.into(),
            enum_values: None,
            default: None,
            minimum: None,
            maximum: None,
            items: Some(Box::new(items)),
        }
    }

    /// Set enum values
    pub fn with_enum(mut self, values: Vec<String>) -> Self {
        self.enum_values = Some(values);
        self
    }

    /// Set default value
    pub fn with_default(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    /// Set minimum value
    pub fn with_minimum(mut self, min: f64) -> Self {
        self.minimum = Some(min);
        self
    }

    /// Set maximum value
    pub fn with_maximum(mut self, max: f64) -> Self {
        self.maximum = Some(max);
        self
    }

    /// Validate a value against this property schema
    pub fn validate(&self, value: &Value) -> ToolResult<()> {
        match self.prop_type.as_str() {
            "string" => {
                if !value.is_string() {
                    return Err(ToolError::ValidationFailed(format!(
                        "Expected string, got {value:?}"
                    )));
                }
                if let Some(enum_values) = &self.enum_values {
                    let val = value.as_str().unwrap();
                    if !enum_values.contains(&val.to_string()) {
                        return Err(ToolError::ValidationFailed(format!(
                            "Value '{val}' not in enum: {enum_values:?}"
                        )));
                    }
                }
            }
            "number" => {
                if !value.is_number() {
                    return Err(ToolError::ValidationFailed(format!(
                        "Expected number, got {value:?}"
                    )));
                }
                let num = value.as_f64().unwrap();
                if let Some(min) = self.minimum {
                    if num < min {
                        return Err(ToolError::ValidationFailed(format!(
                            "Value {num} is less than minimum {min}"
                        )));
                    }
                }
                if let Some(max) = self.maximum {
                    if num > max {
                        return Err(ToolError::ValidationFailed(format!(
                            "Value {num} is greater than maximum {max}"
                        )));
                    }
                }
            }
            "boolean" => {
                if !value.is_boolean() {
                    return Err(ToolError::ValidationFailed(format!(
                        "Expected boolean, got {value:?}"
                    )));
                }
            }
            "array" => {
                if !value.is_array() {
                    return Err(ToolError::ValidationFailed(format!(
                        "Expected array, got {value:?}"
                    )));
                }
                if let Some(items_schema) = &self.items {
                    for item in value.as_array().unwrap() {
                        items_schema.validate(item)?;
                    }
                }
            }
            _ => {
                return Err(ToolError::ValidationFailed(format!(
                    "Unknown type: {}",
                    self.prop_type
                )));
            }
        }

        Ok(())
    }
}

/// Macro to simplify tool schema definition
#[macro_export]
macro_rules! tool_schema {
    (
        name: $name:expr,
        description: $desc:expr,
        parameters: {
            $(
                $param:ident: {
                    type: $type:expr,
                    description: $param_desc:expr
                    $(, required: $required:expr)?
                    $(, default: $default:expr)?
                    $(, enum: $enum:expr)?
                    $(, min: $min:expr)?
                    $(, max: $max:expr)?
                }
            ),* $(,)?
        }
    ) => {
        {
            let mut schema = $crate::ToolSchema::new($name, $desc);
            $(
                let mut prop = match $type {
                    "string" => $crate::PropertySchema::string($param_desc),
                    "number" => $crate::PropertySchema::number($param_desc),
                    "boolean" => $crate::PropertySchema::boolean($param_desc),
                    _ => panic!("Unknown type: {}", $type),
                };
                $(prop = prop.with_default($default);)?
                $(prop = prop.with_enum($enum);)?
                $(prop = prop.with_minimum($min);)?
                $(prop = prop.with_maximum($max);)?

                let required = true $(&& $required)?;
                schema = schema.add_parameter(stringify!($param), prop, required);
            )*
            schema
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_creation() {
        let schema = ToolSchema::new("test_tool", "A test tool")
            .add_parameter("param1", PropertySchema::string("First parameter"), true)
            .add_parameter("param2", PropertySchema::number("Second parameter"), false);

        assert_eq!(schema.name, "test_tool");
        assert_eq!(schema.parameters.required.len(), 1);
        assert_eq!(schema.parameters.properties.len(), 2);
    }

    #[test]
    fn test_validation_success() {
        let schema = ToolSchema::new("test", "test")
            .add_parameter("name", PropertySchema::string("Name"), true)
            .add_parameter("age", PropertySchema::number("Age"), false);

        let args = json!({
            "name": "Alice",
            "age": 30
        });

        assert!(schema.validate(&args).is_ok());
    }

    #[test]
    fn test_validation_missing_required() {
        let schema = ToolSchema::new("test", "test").add_parameter(
            "name",
            PropertySchema::string("Name"),
            true,
        );

        let args = json!({});

        assert!(schema.validate(&args).is_err());
    }

    #[test]
    fn test_validation_type_mismatch() {
        let schema = ToolSchema::new("test", "test").add_parameter(
            "age",
            PropertySchema::number("Age"),
            true,
        );

        let args = json!({
            "age": "not a number"
        });

        assert!(schema.validate(&args).is_err());
    }
}
