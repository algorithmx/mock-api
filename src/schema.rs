use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Root configuration for a mock project
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    /// Project description/name
    pub description: String,
    /// Map of endpoint paths to their configurations
    /// Key: endpoint path (e.g., "/api/test")
    /// Value: endpoint configuration
    pub endpoints: HashMap<String, Endpoint>,
}

/// Configuration for a specific endpoint
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    /// List of conditions to match and their corresponding responses
    /// Multiple conditions allow different responses based on request details
    #[serde(rename = "when")]
    pub conditions: Vec<WhenCondition>,
    #[serde(skip)]
    pub condition_map: HashMap<EndpointKey, (ResponseConfig, u64)>,
}

/// Defines a specific request condition and its response
#[derive(Debug, Serialize, Deserialize, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct QueryParam {
    /// Matching operator: "is", "is!", "contains", "contains!"
    pub operator: String,
    /// Expected value to match against
    pub value: String,
}

/// Endpoint hashmap key
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct EndpointKey {
    pub method: String, // UPPERCASE
    pub queries: Option<HashMap<String, QueryParam>>,
    pub body: Option<serde_json::Value>,
}

/// Response configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
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

// Custom serialization/deserialization for Endpoint to build the HashMap
impl Endpoint {
    pub fn build_condition_map(&mut self) {
        self.condition_map = self.conditions.iter().map(|condition| {
            let key = EndpointKey {
                method: condition.method.clone(),
                queries: condition.request.as_ref().and_then(|req| req.queries.clone()),
                body: condition.request.as_ref().and_then(|req| req.body.clone()),
            };
            let value = (condition.response.clone(), condition.delay);
            (key, value)
        }).collect();
    }
}


// Custom Hash implementation for HashMap in RequestConfig
impl Hash for RequestConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash length first for better distribution
        if let Some(queries) = &self.queries {
            queries.len().hash(state);
            // Combine all key-value hashes in sorted order without allocation
            let mut hasher = DefaultHasher::new();
            for key in queries.keys().collect::<Vec<_>>() {
                key.hash(&mut hasher);
                queries[key].hash(&mut hasher);
            }
            hasher.finish().hash(state);
        }
        
        if let Some(headers) = &self.headers {
            headers.len().hash(state);
            let mut hasher = DefaultHasher::new();
            for key in headers.keys().collect::<Vec<_>>() {
                key.hash(&mut hasher);
                headers[key].hash(&mut hasher);
            }
            hasher.finish().hash(state);
        }
        
        self.body.hash(state);
    }
}

impl Hash for EndpointKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.method.hash(state);
        
        if let Some(queries) = &self.queries {
            // Sort keys before hashing to ensure consistent ordering
            let mut sorted_keys: Vec<_> = queries.keys().collect();
            sorted_keys.sort();
            
            queries.len().hash(state);
            let mut hasher = DefaultHasher::new();
            for key in sorted_keys {
                key.hash(&mut hasher);
                queries[key].hash(&mut hasher);
            }
            hasher.finish().hash(state);
        }
        
        self.body.hash(state);
    }
}