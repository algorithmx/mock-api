use std::{
  collections::HashMap,
  io::Write,
  net::{TcpListener, TcpStream},
  sync::{Arc, Mutex},
};

mod helpers;
mod thread_pool;
pub mod types;

use types::{Request, Response};

pub use thread_pool::ThreadPool;

use self::types::{Method, Nested, RequestOption, RequestPathPattern};

pub struct Listener {
  path: RequestPathPattern,
  method: Method,
  handler: Handler,
}

type Handler = Box<dyn Fn(Request) -> Response + Send + 'static>;

pub struct Server {
  max_connections: usize,
  connection_handler: Arc<Mutex<ConnectionHandler>>,
}

pub struct ServerConf {
  pub max_connections: usize,
}

impl Server {
  pub fn new(conf: ServerConf) -> Server {
    Server {
      max_connections: conf.max_connections,
      connection_handler: Arc::new(Mutex::new(ConnectionHandler::new())),
    }
  }

  pub fn listen(&self, addr: String) {
    // Some possible reasons for binding to fail:
    // - connecting to a port requires administrator privileges.
    // - listening to a port which is occupied.
    let listener = TcpListener::bind(addr).unwrap();

    // Limit the number of threads in the pool to a small number to protect us
    // from Denial of Service (DoS) attacks.
    let pool = ThreadPool::new(self.max_connections);

    for stream in listener.incoming() {
      // The browser signals the end of an HTTP request by sending two newline
      // characters in a row.
      // The reason we might receive errors from the incoming method when a client
      // connects to the server is that we're not actually iterating over
      // connections. Instead, we're iterating over connection attempts. The
      // connection might not be successful for a number of reasons, many of them
      // operating system specific. For example, many operating systems have a
      // limit to the number of simultaneous open connections they can support;
      // new connection attempts beyond that number will produce an error until
      // some of the open connections are closed.
      let stream = stream.unwrap();

      let connection_handler = self.connection_handler.clone();

      pool.execute(move || {
        let connection_handler = connection_handler.lock().unwrap();
        connection_handler.handle_connection(stream);
      });
    }
  }

  pub fn request<F>(&mut self, request_handler: F, option: RequestOption)
  where
    F: Fn(Request) -> Response + Send + 'static,
  {
    let mut connection_handler = self.connection_handler.lock().unwrap();

    connection_handler.listeners.push(Listener {
      method: option.method,
      path: option.path,
      handler: Box::new(request_handler),
    });
  }

  pub fn get<F>(&mut self, path: &str, request_handler: F)
  where
    F: Fn(Request) -> Response + Send + 'static,
  {
    self.request(
      request_handler,
      RequestOption {
        path: RequestPathPattern::Exact(String::from(path)),
        method: Method::Get,
      },
    );
  }

  pub fn post<F>(&mut self, path: &str, request_handler: F)
  where
    F: Fn(Request) -> Response + Send + 'static,
  {
    self.request(
      request_handler,
      RequestOption {
        path: RequestPathPattern::Exact(String::from(path)),
        method: Method::Post,
      },
    );
  }

  pub fn put<F>(&mut self, path: &str, request_handler: F)
  where
    F: Fn(Request) -> Response + Send + 'static,
  {
    self.request(
      request_handler,
      RequestOption {
        path: RequestPathPattern::Exact(String::from(path)),
        method: Method::Put,
      },
    );
  }

  #[cfg(test)]
  pub fn handle_request(&self, request: &Request) -> Response {
    let connection_handler = self.connection_handler.lock().unwrap();
    
    for listener in &connection_handler.listeners {
        if let Some(parsed_path) = helpers::parse_request_path(&listener.path, &request.path) {
            if listener.method.to_string() == request.method {
                let mut request = request.clone();
                request.path = parsed_path.path;
                request.queries = parsed_path.queries;
                request.params = parsed_path.params;
                request.matches = parsed_path.matches;
                return (listener.handler)(request);
            }
        }
    }
    
    // Return 404 if no matching route is found
    let mut body = Nested::new();
    body.insert_string("error".to_string(), "Not Found".to_string());
    Response::json(404, body, None)
  }

  #[cfg(test)]
  pub fn test_request(&self, method: Method, path: &str, headers: Option<HashMap<String, String>>, body: Option<String>) -> Response {
    let request = Request {
      method: method.to_string(),
      path: path.to_string(),
      headers: headers.unwrap_or(HashMap::new()),
      body: body.unwrap_or("".to_string()),
      version: "1.1".to_string(),
      queries: HashMap::new(),
      params: HashMap::new(),
      matches: Vec::new(),
    };
    self.handle_request(&request)
  }
}

impl Response {
  pub fn json(status: u16, body: Nested, headers: Option<HashMap<String, String>>) -> Response {
    let mut headers = headers.unwrap_or(HashMap::new());

    headers.insert(
      String::from("Content-Type"),
      String::from("application/json"),
    );

    Response {
      status,
      body: helpers::stringify_nested(&body),
      headers,
    }
  }

  pub fn ok(body: String, headers: Option<HashMap<String, String>>) -> Response {
    let mut headers = headers.unwrap_or(HashMap::new());

    if headers.get("Content-Type").is_none() {
      headers.insert(String::from("Content-Type"), String::from("text/plain"));
    }

    Response {
      status: 200,
      body,
      headers,
    }
  }
}

struct ConnectionHandler {
  listeners: Vec<Listener>,
}

impl ConnectionHandler {
  pub fn new() -> ConnectionHandler {
    ConnectionHandler {
      listeners: Vec::new(),
    }
  }

  fn dispatch_request(&self, request: Request) -> (u16, String, String) {
    for listener in self.listeners.iter() {
      if let Some(parsed_path) = 
        helpers::parse_request_path(
          &listener.path,
          &request.path
        ) {
        if listener.method.to_string() == request.method {
          let mut request = request;
          request.path = parsed_path.path;
          request.queries = parsed_path.queries;
          request.params = parsed_path.params;
          request.matches = parsed_path.matches;

          let response = (listener.handler)(request);
          let mut response_headers = String::new();

          if !response.headers.is_empty() {
            for (key, value) in response.headers.iter() {
              response_headers.push_str(&format!("{}: {}\r\n", key, value));
            }
          }

          return (response.status, response.body, response_headers);
        }
      }
    }
    
    (404, String::new(), String::new())
  }

  pub fn handle_connection(&self, mut stream: TcpStream) {
    let request = helpers::parse_tcp_stream(&mut stream).unwrap();
    let (response_status, response_body, response_headers) = 
      self.dispatch_request(request);
    let length = response_body.len();
    let response = format!(
      "HTTP/1.1 {response_status}\r\n{response_headers}Content-Length: {length}\r\n\r\n{response_body}"
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
  }
}
