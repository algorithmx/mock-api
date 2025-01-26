use std::{
  collections::HashMap,
  io::Write,
  net::{TcpListener, TcpStream},
  sync::{Arc, Mutex},
  time::Duration,
  sync::atomic::{AtomicBool, Ordering}
};

mod helpers;

pub mod types;

use types::{Request, Response};

use self::types::{Method, Nested, RequestOption, RequestPathPattern};

// multiple threads are not needed, as tokio handles concurrency internally
use tokio::runtime::Runtime;
use tokio::task;

pub struct Listener {
  path: RequestPathPattern,
  method: Method,
  handler: Handler,
}

type Handler = Box<dyn Fn(Request) -> Response + Send + 'static>;


pub struct Server {
  // max_connections: usize, // TODO: remove this
  connection_handler: Arc<Mutex<ConnectionHandler>>,
  running: Arc<AtomicBool>,
}


#[allow(dead_code)]
pub struct ServerConf {
  pub max_connections: usize,
}


impl Server {

  #[allow(unused_variables)]
  pub fn new(conf: ServerConf) -> Server {
    Server {
      // max_connections: conf.max_connections,
      // TODO properly set the max_connections
      connection_handler: Arc::new(Mutex::new(ConnectionHandler::new())),
      running: Arc::new(AtomicBool::new(true)),
    }
  }

  // TODO: remove this
  // pub fn stop(&self) {
  //   self.running.store(false, Ordering::SeqCst);
  // }

  pub fn listen(&self, addr: String) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
      let listener = TcpListener::bind(addr).unwrap();
      
      while self.running.load(Ordering::SeqCst) {
        if let Ok((stream, _)) = listener.accept() {
          let connection_handler = self.connection_handler.clone();
          task::spawn(async move {
            let connection_handler = connection_handler.lock().unwrap();
            connection_handler.handle_connection(stream);
          });
        }
      }
    });
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
            if listener.method == request.method {
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
      method: method.clone(),
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
        if listener.method == request.method {
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
    // Set read timeout to prevent hanging
    stream.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
    
    let result = (|| {
        let request = helpers::parse_tcp_stream(&mut stream)?;
        let (response_status, response_body, response_headers) = 
            self.dispatch_request(request);
        let length = response_body.len();
        let response = format!(
            "HTTP/1.1 {response_status}\r\n{response_headers}Content-Length: {length}\r\n\r\n{response_body}"
        );
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        Ok::<_, std::io::Error>(())
    })();

    if let Err(e) = result {
        eprintln!("Connection error: {}", e);
    }
    
    // Ensure stream is properly closed
    drop(stream);
  }
}
