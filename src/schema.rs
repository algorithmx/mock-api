use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration for a mock project
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project description/name
    pub description: String,
    /// Map of endpoint paths to their configurations
    /// Key: endpoint path (e.g., "/api/test")
    /// Value: endpoint configuration
    pub endpoints: HashMap<String, Endpoint>,
}

/// Configuration for a specific endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    /// List of conditions to match and their corresponding responses
    /// Multiple conditions allow different responses based on request details
    pub when: Vec<WhenCondition>,
}

/// Defines a specific request condition and its response
#[derive(Debug, Serialize, Deserialize)]
pub struct WhenCondition {
    /// HTTP method (GET, POST, PUT, etc.)
    pub method: String,
    /// Request matching criteria (queries, headers, body)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<RequestConfig>,
    /// Response to return when request matches
    pub response: ResponseConfig,
    /// Optional delay in milliseconds before sending response
    #[serde(default)]
    pub delay: u64,
}

/// Request matching configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestConfig {
    /// Map of query parameter names to their matching rules
    /// Key: query parameter name
    /// Value: matching operator and expected value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queries: Option<HashMap<String, QueryParam>>,
    /// Map of expected request headers
    /// Key: header name
    /// Value: expected header value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// Optional JSON body to match against
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

/// Query parameter matching configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryParam {
    /// Matching operator: "is", "is!", "contains", "contains!"
    pub operator: String,
    /// Expected value to match against
    pub value: String,
}

/// Response configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseConfig {
    /// HTTP status code to return
    pub status: u16,
    /// Response headers
    /// Key: header name
    /// Value: header value
    pub headers: HashMap<String, String>,
    /// Optional JSON response body
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}