mod web_server;

use std::{collections::HashMap, fs::read_to_string};

use web_server::{
  types::{Method, Nested, RequestOption, Response},
  Server, ServerConf,
};

mod handlers;
mod helpers;

fn main() {
  let server_addr = format!("127.0.0.1:{}", helpers::get_env_var("PORT", "53500".to_string()));
  let max_connections: usize = helpers::get_env_var("MAX_CONNECTIONS", 1000);

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
      path: web_server::types::RequestPathPattern::Match(r"^/projects/([^/]+)/([^?]+)".to_string()),
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
