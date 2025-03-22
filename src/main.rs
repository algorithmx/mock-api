mod web_server;

use web_server::{
  types::{Method, RequestOption, Response},
  Server, ServerConf,
};

mod llm;

mod schema;
mod handlers;
mod helpers;
mod cache;

fn init() -> (String, usize, String) {
    let server_addr = format!("127.0.0.1:{}", helpers::get_env_var("MOCK_SERVER_PORT", "53500".to_string()));
    let max_connections: usize = helpers::get_env_var("MOCK_SERVER_MAX_CONN", 1000);
    let database_root_folder = helpers::get_env_var("MOCK_SERVER_DB_ROOT", "./database".to_string());
    
    println!("Server is running:");
    println!("  - Address: {}", server_addr.clone());
    println!("  - Database root folder: {}", database_root_folder);
    println!("  - Max connections: {}", max_connections);

    (server_addr, max_connections, database_root_folder)
}

fn main() {
    let (server_addr, max_connections, _) = init();
    
    let mut server = Server::new(ServerConf {
        max_connections: max_connections,
    });

    server.get("/projects/:name", handlers::get_config());

    server.post("/projects/:name", handlers::save_config());

    server.put("/projects/:name", handlers::save_config());

    server.post("/llm/:name", handlers::build_config_with_llm());

    // DO NOT MODIFY THIS STRING r !!!
    let r = r"^/projects/(\w+)((/\w+)+)(\?(\w+=\w+)(&\w+=\w+)*)?$"; 
    // explain: 
    // 1. /projects/ - matches the exact path
    // 2. (\w+) - matches the project name
    // 3. (/\w+)+ - matches the API path (with starting slash, e.g. "/api/v1")
    // 4. (\?(\w+=\w+)(&\w+=\w+)*)? - matches the queries
    // DO NOT MODIFY THIS REGEX STRING r !!!
    server.request(
      handlers::mock_request(),
      RequestOption {
        path: web_server::types::RequestPathPattern::Match(r.to_string()),
        method: Method::Get,
      },
    );
    server.request(
        handlers::mock_request(),
        RequestOption {
          path: web_server::types::RequestPathPattern::Match(r.to_string()),
          method: Method::Post,
        },
    );

    // Add API documentation endpoint
    server.get("/api-doc", |_| {
      let html = include_str!("api-doc.html");
      Response::html(html.to_string())
    });

    server.get("/hello", |_| {
        Response::ok("hello".to_string(), None)
    });

    server.get("/create-project", |_| {
        let html = include_str!("templates/new_project.html");
        Response::html(html.to_string())
    });

    server.listen(server_addr);


}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use temp_env;
    use tempfile::TempDir;
    

    use super::*;

    fn setup_test_server() -> Server {
        let mut server = Server::new(ServerConf {
            max_connections: 10,
        });

        server.get("/projects/:name", handlers::get_config());
        server.post("/projects/:name", handlers::save_config());
        server.put("/projects/:name", handlers::save_config());

        server.request(
            handlers::mock_request(),
            RequestOption {
                path: web_server::types::RequestPathPattern::Match(r"^/projects/([^/]+)/([^?]+)(?:\?(.*))?$".to_string()),
                method: Method::Get,
            },
        );
        server
    }

    #[test]
    fn test_nonexistent_project() {
        let server = setup_test_server();
        let response = server.test_request(Method::Get, "/projects/nonexistent", None, None);
        assert_eq!(response.status, 404);
        assert!(response.body.contains("Project does not exist"));
    }

    #[test]
    fn test_create_and_get_project() {
        let test_dir = TempDir::new().unwrap();
        fs::create_dir_all(test_dir.path().join("projects")).unwrap();
        
        temp_env::with_var("MOCK_SERVER_DB_ROOT", Some(test_dir.path().to_str().unwrap()), || {
            let server = setup_test_server();
            
            // Create test project
            let test_config = r#"{"description": "test", "endpoints": {}}"#;
            let response = server.test_request(Method::Post, "/projects/test-project", None, Some(test_config.to_string()));
            assert_eq!(response.status, 200);

            // Get created project
            let response = server.test_request(Method::Get, "/projects/test-project", None, None);
            assert_eq!(response.status, 200);
            assert!(response.body.contains("endpoints"));
        });
    }

    #[test]
    fn test_mock_endpoint() {
        let test_dir = TempDir::new().unwrap();
        fs::create_dir_all(test_dir.path().join("projects")).unwrap();
        temp_env::with_var("MOCK_SERVER_DB_ROOT", Some(test_dir.path().to_str().unwrap()), || {
            let server = setup_test_server();
            // Create test project with comprehensive mock configuration
            let test_config = r#"{
                "description": "test-mock1",
                "endpoints": {
                    "api/test": {
                        "when": [{
                            "method": "GET",
                            "request": {
                                "queries": {"filter": {"operator": "is", "value": "active"}},
                                "headers": {"x-test": "value"}
                            },
                            "response": {
                                "status": 200,
                                "body": "mocked response",
                                "headers": {"content-type": "text/plain"}
                            },
                            "delay": 0
                        }]
                    }
                }}"#;
            let response1 = server.test_request(Method::Post, "/projects/test-mock1", None, Some(test_config.to_string()));
            assert_eq!(response1.status, 200);
            // Test mock endpoint with matching request
            let mut headers = HashMap::new();
            headers.insert("x-test".to_string(), "value".to_string());
            let response = server.test_request(
                Method::Get, 
                "/projects/test-mock1/api/test?filter=active", 
                Some(headers),
                None
            );
            assert_eq!(response.status, 200);
            assert_eq!(response.body, "mocked response");
            assert_eq!(response.headers.get("content-type").unwrap(), "text/plain");
        });
    }
}
