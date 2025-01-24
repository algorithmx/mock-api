use crate::{helpers, web_server, schema};
use serde_json::Value;
use std::{collections::HashMap, fs, fs::read_to_string};
use web_server::types::{Nested, Request, Response};


pub fn get_config() -> impl Fn(Request) -> Response {
    |request: Request| {
        println!("%%%% get_config request: {:?}", request);
        let file = helpers::config_file_path_from_request(&request);

        if file.exists() {
            let content = read_to_string(file).unwrap();
            let mut headers = HashMap::new();
            headers.insert(
                String::from("Content-Type"),
                String::from("application/json"),
            );
            Response::ok(content, Some(headers))
        } else {
            let mut body = Nested::new();
            body.insert_string("error".to_string(), "Project does not exist.".to_string());
            Response::json(404, body, None)
        }
    }
}


/// Returns a closure that saves a project's config.
pub fn save_config() -> impl Fn(Request) -> Response {
  |request: Request| {
    let file_path = helpers::config_file_path_from_request(&request);

    if request.method == "POST" {
      let mut body = Nested::new();
      if file_path.exists() {
        body.insert_string("error".to_string(), "Project already exists.".to_string());
        return Response::json(400, body, None);
      }
    } else if request.method == "PUT" && !file_path.exists() {
      let mut body = Nested::new();
      body.insert_string("error".to_string(), "Project does not exist.".to_string());
      return Response::json(400, body, None);
    }

    // Write empty string if body is empty/missing
    let content = request.body.as_str();
    // if content is an empty string, return
    println!("%%%% content: {:?}", content);
    fs::write(file_path, content).unwrap();

    let mut body = Nested::new();
    body.insert_string("result".to_string(), "ok".to_string());
    Response::json(200, body, None)
  }
}


pub fn mock_request() -> impl Fn(Request) -> Response {
    |request: Request| {
        // Get project config file path
        let config_path = 
            helpers::get_project_config_file_path(request.matches.get(0).unwrap());
        
        // Check if project exists
        if !config_path.exists() {
            let mut body = Nested::new();
            body.insert_string("error".to_string(), "Project does not exist.".to_string());
            return Response::json(400, body, None);
        }

        // Parse project configuration
        let config_str = match read_to_string(config_path) {
            Ok(content) => content,
            Err(e) => return Response {
                status: 400,
                body: format!("Invalid project configuration file: {}", e),
                headers: HashMap::new(),
            },
        };

        let project_config: schema::ProjectConfig = match serde_json::from_str(&config_str) {
            Ok(config) => config,
            Err(e) => return Response {
                status: 400,
                body: format!("Invalid project configuration format: {}", e),
                headers: HashMap::new(),
            },
        };

        let path = request.matches.get(1).unwrap();
        let method = request.method.to_uppercase();

        // Find matching endpoint and condition
        if let Some(endpoint) = project_config.endpoints.get(path) {
            for condition in &endpoint.when {
                if condition.method.to_uppercase() == method {
                    // Make query/header/body checks optional based on whether they're specified
                    if check_condition(&request, condition, true) { // Pass strictness flag
                        // Apply configured delay
                        if condition.delay > 0 {
                            std::thread::sleep(std::time::Duration::from_millis(condition.delay));
                        }

                        // Convert response body to string, handling null properly
                        let body = condition.response.body
                            .as_ref()
                            .map(|v| if let Value::String(s) = v { s.clone() } else { v.to_string() })
                            .unwrap_or("null".to_string());

                        return Response {
                            status: condition.response.status,
                            body,
                            headers: condition.response.headers.clone(),
                        };
                    }
                }
            }
        }

        // No matching endpoint found
        Response {
            status: 400,
            body: "Not implemented.".to_string(),
            headers: HashMap::new(),
        }
    }
}

fn check_condition(request: &Request, condition: &schema::WhenCondition, strict: bool) -> bool {
    // Only check queries if they're specified
    if !condition.request.queries.is_empty() && !check_queries(request, condition) {
        return false;
    }
    
    // Only check headers if they're specified
    if !condition.request.headers.is_empty() && !check_headers(request, condition) {
        return false;
    }
    
    // Only check body if it's specified
    if condition.request.body.is_some() && !check_body(request, condition, strict) {
        return false;
    }
    
    true
}

fn check_queries(request: &Request, condition: &schema::WhenCondition) -> bool {
    for (expected_query_name, expected_query_param) in &condition.request.queries {
        let actual_query_value = request.queries.get(expected_query_name);
        if actual_query_value.is_none() {
            return false; // Expected query param is missing
        }
        let actual_query_value = actual_query_value.unwrap();

        match expected_query_param.operator.as_str() {
            "is" => {
                if actual_query_value != &expected_query_param.value {
                    return false;
                }
            }
            "is!" => {
                if actual_query_value == &expected_query_param.value {
                    return false;
                }
            }
            "contains" => {
                if !actual_query_value.contains(&expected_query_param.value) {
                    return false;
                }
            }
            "contains!" => {
                if actual_query_value.contains(&expected_query_param.value) {
                    return false;
                }
            }
            op => {
                eprintln!("Warning: Unknown query operator '{}'", op); // Optional: Log unknown operator
                return false; // Or decide to ignore and treat as non-matching
            }
        }
    }
    true // All expected queries matched
}

fn check_headers(request: &Request, condition: &schema::WhenCondition) -> bool {
    for (expected_header_name, expected_header_value) in &condition.request.headers {
        let actual_header_value = request.headers.get(expected_header_name);
        if actual_header_value.is_none() {
            return false; // Expected header is missing
        }
        if actual_header_value.unwrap() != expected_header_value {
            return false; // Header value does not match
        }
    }
    true // All expected headers matched
}

fn check_body(request: &Request, condition: &schema::WhenCondition, strict: bool) -> bool {
    match &condition.request.body {
        None => request.body.is_empty(),
        Some(expected_body) => {
            match serde_json::from_str::<Value>(&request.body) {
                Ok(actual_body_json) => {
                    if strict {
                        &actual_body_json == expected_body
                    } else {
                        // In non-strict mode, only check if the expected fields exist
                        if let Value::Object(expected_obj) = expected_body {
                            if let Value::Object(actual_obj) = actual_body_json {
                                return expected_obj.iter().all(|(k, v)| 
                                    actual_obj.get(k).map_or(false, |av| av == v)
                                );
                            }
                        }
                        false
                    }
                },
                Err(_) => false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web_server::types::Request;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    // Helper function to create test directory structure
    fn setup_test_dir() -> TempDir {
        let test_dir = TempDir::new().unwrap();
        fs::create_dir_all(test_dir.path().join("projects")).unwrap();
        test_dir
    }

    fn create_test_request(method: &str, path: &str, body: Option<String>) -> Request {
        Request {
            method: method.to_string(),
            path: path.to_string(),
            version: "1.1".to_string(),
            headers: HashMap::new(),
            body: body.unwrap_or_default(),
            queries: HashMap::new(),
            params: HashMap::new(),
            matches: Vec::new(),
        }
    }

    #[test]
    fn test_get_config_existing_file() {
        let test_dir = setup_test_dir();
        let project_path = test_dir.path().join("projects").join("test.json");
        fs::write(&project_path, r#"{"test": "data"}"#).unwrap();

        let mut request = create_test_request("GET", "/projects/test", None);
        request.params.insert("name".to_string(), "test".to_string());

        // Set environment variable for test
        temp_env::with_var("MOCK_SERVER_DB_ROOT", Some(test_dir.path().to_str().unwrap()), || {
            let handler = get_config();
            let response = handler(request);

            assert_eq!(response.status, 200);
            assert_eq!(response.body, r#"{"test": "data"}"#);
            assert_eq!(response.headers.get("Content-Type"), Some(&"application/json".to_string()));
        });
    }

    #[test]
    fn test_get_config_nonexistent_file() {
        let mut request = create_test_request("GET", "/projects/nonexistent", None);
        request.params.insert("name".to_string(), "nonexistent".to_string());
        let handler = get_config();
        let response = handler(request);

        assert_eq!(response.status, 404);
        assert!(response.body.contains("Project does not exist"));
    }

    #[test]
    fn test_save_config_post_new_project() {
        let test_dir = setup_test_dir();
        let config_data = r#"{"test": "data"}"#;

        let mut request = create_test_request("POST", "/projects/test", Some(config_data.to_string()));
        request.params.insert("name".to_string(), "test".to_string());

        temp_env::with_var("MOCK_SERVER_DB_ROOT", Some(test_dir.path().to_str().unwrap()), || {
            let handler = save_config();
            let response = handler(request);

            assert_eq!(response.status, 200);
            assert!(response.body.contains("ok"));
            
            let saved_content = fs::read_to_string(
                test_dir.path().join("projects").join("test.json")
            ).unwrap();
            assert_eq!(saved_content, config_data);
        });
    }

    #[test]
    fn test_save_config_post_existing_project() {
        let test_dir = setup_test_dir();
        let project_path = test_dir.path().join("projects").join("test.json");
        fs::write(&project_path, r#"{"existing": "data"}"#).unwrap();

        let mut request = create_test_request("POST", "/projects/test", Some(r#"{"new": "data"}"#.to_string()));
        request.params.insert("name".to_string(), "test".to_string());

        temp_env::with_var("MOCK_SERVER_DB_ROOT", Some(test_dir.path().to_str().unwrap()), || {
            let handler = save_config();
            let response = handler(request);

            assert_eq!(response.status, 400);
            assert!(response.body.contains("Project already exists"));
        });
    }

    #[test]
    fn test_save_config_put_nonexistent_project() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut request = create_test_request("PUT", "/projects/test", Some(r#"{"new": "data"}"#.to_string()));
        request.params.insert("name".to_string(), path.to_str().unwrap().to_string());

        let handler = save_config();
        let response = handler(request);

        assert_eq!(response.status, 400);
        assert!(response.body.contains("Project does not exist"));
    }

    #[test]
    fn test_save_config_put_existing_project() {
        let test_dir = setup_test_dir();
        let project_path = test_dir.path().join("projects").join("test.json");
        fs::write(&project_path, r#"{"existing": "data"}"#).unwrap();

        let new_data = r#"{"new": "data"}"#;
        let mut request = create_test_request("PUT", "/projects/test", Some(new_data.to_string()));
        request.params.insert("name".to_string(), "test".to_string());

        temp_env::with_var("MOCK_SERVER_DB_ROOT", Some(test_dir.path().to_str().unwrap()), || {
            let handler = save_config();
            let response = handler(request);

            assert_eq!(response.status, 200);
            assert!(response.body.contains("ok"));
            assert_eq!(fs::read_to_string(project_path).unwrap(), new_data);
        });
    }

    #[test]
    fn test_mock_request_project_not_found() {
        let test_dir = setup_test_dir();
        
        let mut request = create_test_request("GET", "/projects/nonexistent/api/test", None);
        request.matches = vec!["nonexistent".to_string(), "api/test".to_string()];

        temp_env::with_var("MOCK_SERVER_DB_ROOT", Some(test_dir.path().to_str().unwrap()), || {
            let handler = mock_request();
            let response = handler(request);

            assert_eq!(response.status, 400);
            assert!(response.body.contains("Project does not exist"));
        });
    }

    #[test]
    fn test_mock_request_invalid_config_format() {
        let test_dir = setup_test_dir();
        let project_path = test_dir.path().join("projects").join("test.json");
        fs::write(&project_path, "invalid json").unwrap();

        let mut request = create_test_request("GET", "/projects/test/api/test", None);
        request.matches = vec!["test".to_string(), "api/test".to_string()];

        temp_env::with_var("MOCK_SERVER_DB_ROOT", Some(test_dir.path().to_str().unwrap()), || {
            let handler = mock_request();
            let response = handler(request);

            assert_eq!(response.status, 400);
            assert!(response.body.contains("Invalid project configuration format"));
        });
    }

    #[test]
    fn test_mock_request_no_matching_endpoint() {
        let test_dir = setup_test_dir();
        let project_path = test_dir.path().join("projects").join("test.json");
        fs::write(&project_path, r#"{"description": "test", "endpoints": {}}"#).unwrap();

        let mut request = create_test_request("GET", "/projects/test/api/test", None);
        request.matches = vec!["test".to_string(), "api/test".to_string()];

        temp_env::with_var("MOCK_SERVER_DB_ROOT", Some(test_dir.path().to_str().unwrap()), || {
            let handler = mock_request();
            let response = handler(request);

            assert_eq!(response.status, 400);
            assert_eq!(response.body, "Not implemented.");
        });
    }

    #[test]
    fn test_check_condition_all_pass() {
        let condition = schema::WhenCondition {
            method: "GET".to_string(),
            request: schema::RequestConfig {
                queries: HashMap::new(),
                headers: HashMap::new(),
                body: None,
            },
            response: schema::ResponseConfig {
                status: 200,
                headers: HashMap::new(),
                body: None,
            },
            delay: 0,
        };

        let request = create_test_request("GET", "/test", None);
        assert!(check_condition(&request, &condition, true));
    }

    #[test]
    fn test_check_queries_matching() {
        let mut queries = HashMap::new();
        queries.insert("filter".to_string(), schema::QueryParam {
            operator: "is".to_string(),
            value: "active".to_string(),
        });

        let condition = schema::WhenCondition {
            method: "GET".to_string(),
            request: schema::RequestConfig {
                queries,
                headers: HashMap::new(),
                body: None,
            },
            response: schema::ResponseConfig {
                status: 200,
                headers: HashMap::new(),
                body: None,
            },
            delay: 0,
        };

        let mut request = create_test_request("GET", "/test?filter=active", None);
        request.queries.insert("filter".to_string(), "active".to_string());
        assert!(check_queries(&request, &condition));
    }

    #[test]
    fn test_check_headers_matching() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let condition = schema::WhenCondition {
            method: "GET".to_string(),
            request: schema::RequestConfig {
                queries: HashMap::new(),
                headers,
                body: None,
            },
            response: schema::ResponseConfig {
                status: 200,
                headers: HashMap::new(),
                body: None,
            },
            delay: 0,
        };

        let mut request = create_test_request("GET", "/test", None);
        request.headers.insert("content-type".to_string(), "application/json".to_string());
        assert!(check_headers(&request, &condition));
    }

    #[test]
    fn test_check_body_strict_matching() {
        let body = Some(serde_json::json!({"key": "value"}));

        let condition = schema::WhenCondition {
            method: "POST".to_string(),
            request: schema::RequestConfig {
                queries: HashMap::new(),
                headers: HashMap::new(),
                body,
            },
            response: schema::ResponseConfig {
                status: 200,
                headers: HashMap::new(),
                body: None,
            },
            delay: 0,
        };

        let request = create_test_request("POST", "/test", Some(r#"{"key": "value"}"#.to_string()));
        assert!(check_body(&request, &condition, true));
    }
}
