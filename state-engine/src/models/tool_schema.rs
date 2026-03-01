//! Tool Schema Registry
//!
//! Defines schemas for available tools, enabling input validation and discovery.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::EntityBase;

/// Tool parameter definition for schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<Value>,
    pub enum_values: Option<Vec<Value>>,
}

/// Parameter type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    #[default]
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
}

/// Tool schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub base: EntityBase,
    pub tool_name: String,
    pub description: String,
    pub version: String,
    pub parameters: Vec<ToolParameter>,
    pub return_type: Option<ParameterType>,
    pub return_description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub deprecated: bool,
    pub deprecation_message: Option<String>,
}

impl ToolSchema {
    pub fn new(tool_name: String, description: String) -> Self {
        Self {
            base: EntityBase::new(),
            tool_name,
            description,
            version: "1.0.0".to_string(),
            parameters: Vec::new(),
            return_type: None,
            return_description: None,
            category: None,
            tags: Vec::new(),
            enabled: true,
            deprecated: false,
            deprecation_message: None,
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    pub fn with_parameter(mut self, param: ToolParameter) -> Self {
        self.parameters.push(param);
        self
    }

    pub fn with_return_type(mut self, return_type: ParameterType, description: String) -> Self {
        self.return_type = Some(return_type);
        self.return_description = Some(description);
        self
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn validate_input(&self, input: &Value) -> Result<(), String> {
        // Check if input is an object
        let input_obj = input.as_object()
            .ok_or_else(|| "Input must be a JSON object".to_string())?;

        // Validate required parameters
        for param in &self.parameters {
            if param.required && !input_obj.contains_key(&param.name) {
                return Err(format!("Missing required parameter: {}", param.name));
            }
        }

        // Validate parameter types
        for (key, value) in input_obj {
            if let Some(param) = self.parameters.iter().find(|p| &p.name == key) {
                Self::validate_type(value, &param.param_type, &param.name)?;
                
                // Validate enum values if specified
                if let Some(ref enum_vals) = param.enum_values {
                    if !enum_vals.contains(value) {
                        return Err(format!(
                            "Parameter '{}' value {:?} not in allowed values: {:?}",
                            param.name, value, enum_vals
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_type(value: &Value, expected_type: &ParameterType, name: &str) -> Result<(), String> {
        let valid = match expected_type {
            ParameterType::String => value.is_string(),
            ParameterType::Integer => value.is_i64() || value.is_u64(),
            ParameterType::Number => value.is_number(),
            ParameterType::Boolean => value.is_boolean(),
            ParameterType::Array => value.is_array(),
            ParameterType::Object => value.is_object(),
        };

        if valid {
            Ok(())
        } else {
            Err(format!(
                "Parameter '{}' type mismatch: expected {:?}",
                name, expected_type
            ))
        }
    }
}

/// Retry policy configuration for automatic retries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay_ms: u32,
    pub max_delay_ms: u32,
    pub backoff_multiplier: f32,
    pub retryable_errors: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            retryable_errors: vec![
                "timeout".to_string(),
                "connection_error".to_string(),
                "rate_limit".to_string(),
                "service_unavailable".to_string(),
            ],
        }
    }
}

impl RetryPolicy {
    pub fn calculate_delay(&self, attempt: u32) -> u32 {
        let delay = (self.initial_delay_ms as f32 * self.backoff_multiplier.powi(attempt as i32 - 1)) as u32;
        delay.min(self.max_delay_ms)
    }

    pub fn should_retry(&self, error: &str, attempt: u32) -> bool {
        if attempt >= self.max_attempts {
            return false;
        }
        
        let error_lower = error.to_lowercase();
        self.retryable_errors.iter().any(|e| error_lower.contains(&e.to_lowercase()))
    }
}

/// Tool execution with retry tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryState {
    pub attempt_count: u32,
    pub last_error: Option<String>,
    pub next_retry_at: Option<DateTime<Utc>>,
}

impl Default for RetryState {
    fn default() -> Self {
        Self {
            attempt_count: 0,
            last_error: None,
            next_retry_at: None,
        }
    }
}
