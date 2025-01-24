mod web_server;

use std::{collections::HashMap, fs::read_to_string};

use web_server::{
  types::{Method, Nested, RequestOption, Response},
  Server, ServerConf,
};

mod schema;
mod handlers;
mod helpers;

fn main() {
  let server_addr = format!("127.0.0.1:{}", helpers::get_env_var("MOCK_SERVER_PORT", "53500".to_string()));
  let max_connections: usize = helpers::get_env_var("MOCK_SERVER_MAX_CONN", 1000);
  let database_root_folder = helpers::get_env_var("MOCK_SERVER_DB_ROOT", "./database".to_string());
  println!("Server is running:");
  println!("  - Address: {}", server_addr.clone());
  println!("  - Database root folder: {}", database_root_folder);
  println!("  - Max connections: {}", max_connections);


  let mut server = Server::new(ServerConf {
    max_connections: max_connections,
  });

  server.get("/", |_| {
    let status = 200;
    let headers = None;
    let mut body = Nested::new();
    body.insert_string("name".to_string(), "Hello world!".to_string());

    Response::json(status, body, headers)
  });

  // Get a project.
  server.get("/projects/:name", |request| {
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
  });

  // Create a project.
  server.post("/projects/:name", handlers::save_config());

  // Update a project.
  server.put("/projects/:name", handlers::save_config());

  
  // Registers a route handler for mocking HTTP requests based on project configurations.
  // 
  // This handler matches requests against the pattern: `/projects/{project_name}/{path}`
  // where:
  // - `project_name`: Name of the project containing mock configurations
  // - `path`: The API endpoint path to mock
  //
  // # Request Pattern
  // - URL Pattern: `^/projects/([^/]+)/([^?]+)`
  // - Method: GET
  //
  // # Mock Configuration
  // The mock configurations should be stored in JSON files with the following structure:
  // ```json
  // {
  //   "endpoints": [{
  //     "path": "/api/users",
  //     "when": [{
  //       "method": "GET",
  //       "delay": 1000,
  //       "response": {
  //         "status": 200,
  //         "body": "Response content",
  //         "headers": {
  //           "Content-Type": "application/json"
  //         }
  //       }
  //     }]
  //   }]
  // }
  // ```
  //
  // # Example Usage
  // ```
  // GET /projects/my-project/api/users
  // ```
  // This will look for a mock configuration in my-project's config file
  // matching the path "/api/users" with GET method.
  //
  // # Response
  // - Returns the configured mock response if found
  // - Returns 400 if project doesn't exist
  // - Returns 400 "Not implemented" if no matching endpoint configuration
  server.request(
    handlers::mock_request(),
    RequestOption {
      path: web_server::types::RequestPathPattern::Match(r"^/projects/([^/]+)/([^?]+)\?(\w+=\w+)(&\w+=\w+)*$".to_string()),
      method: Method::Get,
    },
  );

  // Add API documentation endpoint
  server.get("/api-doc", |_| {
    let html = include_str!("api-doc.html");
    Response::html(html.to_string())
  });

  server.listen(server_addr);


}


#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_server() -> Server {
        let mut server = Server::new(ServerConf {
            max_connections: 10,
        });

        // Register all routes same as main()
        server.get("/", |_| {
            let mut body = Nested::new();
            body.insert_string("name".to_string(), "Hello world!".to_string());
            Response::json(200, body, None)
        });
        
        server.get("/projects/:name", |request| {
            let file = helpers::config_file_path_from_request(&request);
            if file.exists() {
                let content = read_to_string(file).unwrap();
                let mut headers = HashMap::new();
                headers.insert(String::from("Content-Type"), String::from("application/json"));
                Response::ok(content, Some(headers))
            } else {
                let mut body = Nested::new();
                body.insert_string("error".to_string(), "Project does not exist.".to_string());
                Response::json(404, body, None)
            }
        });

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
    fn test_root_endpoint() {
        let server = setup_test_server();
        let response = server.test_request(Method::Get, "/", None);
        assert_eq!(response.status, 200);
        assert!(response.body.contains("Hello world!"));
    }

    #[test]
    fn test_nonexistent_project() {
        let server = setup_test_server();
        let response = server.test_request(Method::Get, "/projects/nonexistent", None);
        assert_eq!(response.status, 404);
        assert!(response.body.contains("Project does not exist"));
    }

    #[test]
    fn test_create_and_get_project() {
        let server = setup_test_server();
        
        // Create test project
        let test_config = r#"{"endpoints":[{"path":"/test","when":[{"method":"GET","response":{"status":200,"body":"test"}}]}]}"#;
        let response = server.test_request(Method::Post, "/projects/test-project", Some(test_config.to_string()));
        assert_eq!(response.status, 200);

        // Get created project
        let response = server.test_request(Method::Get, "/projects/test-project", None);
        assert_eq!(response.status, 200);
        assert!(response.body.contains("endpoints"));

        // Cleanup
        // pass
    }

    #[test]
    fn test_mock_endpoint() {
        let server = setup_test_server();
        
        // Create test project with comprehensive mock configuration
        let test_config = r#"{
            "description": "test-mock",
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
        let _ = server.test_request(Method::Post, "/projects/test-mock", Some(test_config.to_string()));

        // Test mock endpoint with matching request
        let mut headers = HashMap::new();
        headers.insert("x-test".to_string(), "value".to_string());
        let response = server.test_request(
            Method::Get, 
            "/projects/test-mock/api/test?filter=active", 
            None
        );
        println!("response: {:?}", response.clone());
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "mocked response");
        assert_eq!(response.headers.get("content-type").unwrap(), "text/plain");
    }
}
