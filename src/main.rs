mod web_server;

use std::{collections::HashMap, fs::read_to_string};
use web_server::{
  types::{Method, Nested, RequestOption, Response},
  Server, ServerConf,
};

mod handlers;
mod helpers;

const SERVER_ADDR: &str = "127.0.0.1:53500";
const MAX_CONNECTIONS: usize = 1000;

fn main() {
  let mut server = Server::new(ServerConf {
    max_connections: MAX_CONNECTIONS,
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
    let html = r#"
      <!DOCTYPE html>
      <html>
      <head>
        <title>Mock API Documentation</title>
        <style>
          body { font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
          .endpoint { margin: 20px 0; padding: 10px; border: 1px solid #ddd; }
          code { background: #f5f5f5; padding: 2px 4px; }
        </style>
      </head>
      <body>
        <h1>Mock API Documentation</h1>
        
        <div class="endpoint">
          <h3>GET /</h3>
          <p>Returns a simple hello world message</p>
          <p>Response: <code>{"name": "Hello world!"}</code></p>
        </div>

        <div class="endpoint">
          <h3>GET /projects/:name</h3>
          <p>Get a project's configuration</p>
          <p>Returns the project configuration file if it exists, otherwise returns 404</p>
        </div>

        <div class="endpoint">
          <h3>POST /projects/:name</h3>
          <p>Create a new project configuration</p>
          <p>Creates a new project if it doesn't exist, returns 400 if project already exists</p>
        </div>

        <div class="endpoint">
          <h3>PUT /projects/:name</h3>
          <p>Update an existing project configuration</p>
          <p>Updates project if it exists, returns 400 if project doesn't exist</p>
        </div>

        <div class="endpoint">
          <h3>GET /projects/:project_name/:path</h3>
          <p>Mock an API endpoint based on project configuration</p>
          <p>Returns mock response based on matching configuration in project file</p>
          <p>Example: <code>GET /projects/my-project/api/users</code></p>
        </div>
      </body>
      </html>
    "#.to_string();

    Response::html(html)
  });

  server.listen(String::from(SERVER_ADDR));
}
