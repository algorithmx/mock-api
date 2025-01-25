use crate::{helpers::{self, get_project_config_file_path}, llm::compose_config, schema, web_server};
use serde_json::Value;
use std::{collections::HashMap, fs, fs::read_to_string};
use web_server::types::{Nested, Request, Response};


pub fn get_config() -> impl Fn(Request) -> Response {
    |request: Request| {
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
    match fs::write(file_path, content) {
        Ok(_) => {
            let mut body = Nested::new();
            body.insert_string("result".to_string(), "ok".to_string());
            Response::json(200, body, None)
        }
        Err(e) => {
            eprintln!("Failed to write config: {}", e);
            let mut body = Nested::new();
            body.insert_string("error".to_string(), format!("Failed to save config: {}", e));
            Response::json(500, body, None)
        }
    }
  }
}


pub fn build_config_with_llm() -> impl Fn(Request) -> Response {
    |request: Request| {
        let project_name = match request.params.get("name") {
            Some(name) => name,
            None => {
                let mut body = Nested::new();
                body.insert_string("error".to_string(), "Project name missing.".to_string());
                return Response::json(400, body, None);
            }
        };
        let config_file_path = get_project_config_file_path(project_name);

        let config = match compose_config(&request.body, &project_name) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to compose config: {}", e);
                let mut body = Nested::new();
                body.insert_string("error".to_string(), "Invalid configuration received from LLM.".to_string());
                return Response::json(400, body, None);
            }
        };

        let project_config: schema::ProjectConfig = match serde_json::from_str(&config) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Deserialization error (schema validation): {}", e);
                let mut body = Nested::new();
                body.insert_string("error".to_string(), "Configuration json received from LLM is not compatible with the schema.".to_string());
                return Response::json(400, body, None);
            }
        };

        let config_value: Nested = match serde_json::to_value(&project_config) {
            Ok(value) => {
                match serde_json::from_value(value) {
                    Ok(nested) => nested,
                    Err(e) => {
                        eprintln!("Failed to convert ProjectConfig to Nested: {}", e);
                        let mut body = Nested::new();
                        body.insert_string("error".to_string(), "Internal server error".to_string());
                        return Response::json(500, body, None);
                    }
                }
            },
            Err(e) => {
                eprintln!("Serialization error (ProjectConfig to serde_json::Value): {}", e);
                let mut body = Nested::new();
                body.insert_string("error".to_string(), "Internal server error".to_string());
                return Response::json(500, body, None);
            }
        };

        if let Err(e) = fs::write(config_file_path, config) {
            eprintln!("Failed to write config file: {}", e);
            let mut body = Nested::new();
            body.insert_string("error".to_string(), format!("Failed to save config: {}", e));
            return Response::json(500, body, None);
        }

        Response::json(200, config_value, None)
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
            status: 406,
            body: "Not implemented.".to_string(),
            headers: HashMap::new(),
        }
    }
}


/// Check if the request matches the condition.
fn check_condition(request: &Request, condition: &schema::WhenCondition, strict: bool) -> bool {
    //! DO NOT MODIFY THIS FUNCTION
    match &condition.request {
        Some(cond_req) => {
            if !check_queries(&request.queries, &cond_req.queries) {
                return false;
            }
            if !check_headers(&request.headers, &cond_req.headers) {
                return false;
            }
            if !check_body(&request.body, &cond_req.body, strict) {
                return false;
            }
            return true;
        }
        None => {
            // check all of the request.queries, request.headers, request.body are empty
            // removed && request.headers.is_empty()
            return request.queries.is_empty() && request.body.is_empty();
        }
    }
}


/// Check if the request queries match the condition's request queries.
fn check_queries(request_queries: &HashMap<String, String>, queries_from_cond_req: &Option<HashMap<String, schema::QueryParam>>) -> bool {
    //! DO NOT MODIFY THIS FUNCTION
    match queries_from_cond_req {
        Some(cond_req_queries) => {
            for (expected_query_name, expected_query_param) in cond_req_queries {
                if request_queries.is_empty() {
                    return false;
                }
                let actual_query_value = request_queries.get(expected_query_name);
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
            return true; // All expected queries matched
        }
        None => {
            return request_queries.is_empty();
        }
    }
}


/// Check if the request headers match the condition's request headers.
fn check_headers(request_headers: &HashMap<String, String>, headers_from_cond_req: &Option<HashMap<String, String>>) -> bool {
    //! DO NOT MODIFY THIS FUNCTION
    match headers_from_cond_req {
        Some(cond_req_headers) => {
            for (expected_header_name, expected_header_value) in cond_req_headers {
                let actual_header_value = request_headers.get(expected_header_name);
                if actual_header_value.is_none() {
                    return false; // Expected header is missing
                }
                let actual_header_value = actual_header_value.unwrap();
                if actual_header_value != expected_header_value {
                    return false; // Header value does not match
                }
            }
            return true; // All expected headers matched
        }
        None => {
            return request_headers.is_empty();
        }
    }
}


/// Check if the request body matches the condition's request body.
fn check_body(request_body: &String, body_from_cond_req: &Option<Value>, strict: bool) -> bool {
    //! DO NOT MODIFY THIS FUNCTION
    match body_from_cond_req {
        None => request_body.is_empty(),
        Some(expected_body) => {
            match serde_json::from_str::<Value>(&request_body) {
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

            assert_eq!(response.status, 406);
            assert_eq!(response.body, "Not implemented.");
        });
    }

    #[test]
    fn test_check_condition_all_pass() {
        let condition = schema::WhenCondition {
            method: "GET".to_string(),
            request: Some(schema::RequestConfig {
                queries: None,
                headers: None,
                body: None,
            }),
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
        let mut request_queries = HashMap::new();
        request_queries.insert("filter".to_string(), "active".to_string());

        let mut cond_req_queries = HashMap::new();
        cond_req_queries.insert("filter".to_string(), schema::QueryParam {
            operator: "is".to_string(),
            value: "active".to_string(),
        });

        let condition_request = schema::RequestConfig {
            queries: Some(cond_req_queries),
            headers: None,
            body: None,
        };

        assert!(check_queries(&request_queries, &condition_request.queries));
    }

    #[test]
    fn test_check_headers_matching() {
        let mut request_headers = HashMap::new();
        request_headers.insert("content-type".to_string(), "application/json".to_string());

        let mut cond_req_headers = HashMap::new();
        cond_req_headers.insert("content-type".to_string(), "application/json".to_string());

        let condition_request = schema::RequestConfig {
            queries: None,
            headers: Some(cond_req_headers),
            body: None,
        };

        assert!(check_headers(&request_headers, &condition_request.headers));
    }

    #[test]
    fn test_check_body_strict_matching() {
        let request_body = r#"{"key": "value"}"#.to_string();
        let condition_request = schema::RequestConfig {
            queries: None,
            headers: None,
            body: Some(serde_json::json!({"key": "value"})),
        };

        assert!(check_body(&request_body, &condition_request.body, true));
    }
}
