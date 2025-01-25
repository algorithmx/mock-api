use regex::Regex;

use std::{
  collections::HashMap,
  io::{BufRead, BufReader, Error as IoError, Read},
  net::TcpStream,
};

use super::types::{Nested, NestedValue, Request, RequestPath, RequestPathPattern};


/// Converts a [Nested] to a JSON string.
pub fn stringify_nested(nested: &Nested) -> String {
  let mut result = String::new();
  result.push_str("{ ");
  for (i, (key, value)) in nested.iter().enumerate() {
    result.push_str(&format!("\"{}\": {}", key, stringfy_nested_value(value)));
    if i < nested.len() - 1 {
      result.push_str(", ");
    }
  }
  result.push_str(" }");
  result
}


/// Converts a [NestedValue] to a JSON string.
fn stringfy_nested_value(nested: &NestedValue) -> String {
  match nested {
    NestedValue::Map(nested) => stringify_nested(nested),
    NestedValue::Str(value) => format!("\"{}\"", value),
    NestedValue::Bool(value) => format!("{}", value),
    NestedValue::Int(value) => format!("{}", value),
    NestedValue::Float(value) => format!("{}", value),
    NestedValue::Array(value) => {
        let elements: Vec<String> = value.iter()
            .map(|v| stringfy_nested_value(v))
            .collect();
        format!("[{}]", elements.join(", "))
    },
  }
}


/// Extracts queries from a query string.
fn extract_queries(query_string: &str) -> HashMap<String, String> {
  query_string
  .split('&')
  .filter_map(|pair| {
      let mut parts = pair.split('=');
      match (parts.next(), parts.next()) {
          (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
          _ => None
      }
  })
  .collect()
}


/// Splits request path into path and query parameters
fn split_path_and_queries(request_path: &str) -> (&str, HashMap<String, String>) {
    match request_path.find('?') {
      None => (request_path, HashMap::new()),
      Some(query_start) => {
        let query_string = &request_path[query_start + 1..];
        let queries = extract_queries(query_string);
        (&request_path[..query_start], queries)
      },
    }
}


/// Compares the number of slashes in a pattern and a path.
fn compare_slash_counts(pattern: &str, path: &str) -> bool {
  let number_of_slashes_in_pattern = pattern.matches('/').count();
  let number_of_slashes_in_path = path.matches('/').count();
  number_of_slashes_in_pattern == number_of_slashes_in_path
} 


/// Handles exact path matching with parameters
fn handle_exact_path(pattern: &str, path: &str, queries: &HashMap<String, String>) -> Option<RequestPath> {
  if !queries.is_empty() {
    // exact match does not allow queries
    return None;
  }
  // count the number of slashes "/" in the pattern and path
  // if the number of slashes is not the same, then the path does not match the pattern => return None
  if !compare_slash_counts(pattern, path) {
    return None;
  }
  let pattern_segments: Vec<&str> = pattern.split('/').collect();
  let path_segments: Vec<&str> = path.split('/').collect();
  if pattern_segments.len() != path_segments.len() {
    return None;
  }
  let mut params = HashMap::new();
  for (pattern, request) in pattern_segments.iter().zip(path_segments.iter()) {
      if pattern.starts_with(':') {
          params.insert(pattern[1..].to_string(), request.to_string());
      } else if pattern != request {
          return None;
      }
  }
  Some(RequestPath {
      path: path.to_string(),
      queries: HashMap::new(),
      params,
      matches: Vec::new(),
  })
}


/// Constructs the matches from a regex pattern.
fn construct_matches(regex: &Regex, full_path: &str) -> Option<Vec<String>> {
  let captures = regex.captures(full_path)?;
  let matches: Vec<String> = captures
        .iter()
        .skip(1)
        .filter_map(|c| c.map(|m| m.as_str().to_string()))
        .collect();
  Some(matches)
}

/// Handles regex pattern matching
fn handle_regex_path(pattern: &str, full_path: &str, path: &str, queries: &HashMap<String, String>) -> Option<RequestPath> {
    // from main.rs
    // currently we only support one level of API path
    // let r = r"^/projects/(\w+)(/\w+)+(\?(\w+=\w+)(&\w+=\w+)*)?$";  // <-- pattern
    // explain: 
    // 1. /projects/ - matches the exact path
    // 2. (\w+) - matches the project name (matches[0])
    // 3. (/\w+)+ - matches the API path (with starting slash, e.g. "/api/v1") (matches[1])
    // 4. (\?(\w+=\w+)(&\w+=\w+)*)? - matches the queries (matches[2])
    let regexp = Regex::new(pattern).ok()?;
    let matches = construct_matches(&regexp, full_path)?;
    Some(RequestPath {
        path: path.to_string(),
        queries: queries.clone(),
        params: HashMap::new(),
        matches,
    })
}


/// Parses the parameters in a path.
/// Return None if the path does not match the pattern.
/// Return Some(RequestPath) if the path matches the pattern.
pub fn parse_request_path(
    path_pattern: &RequestPathPattern,
    request_path: &str,
) -> Option<RequestPath> {
    let (path, queries) = 
      split_path_and_queries(request_path);
    match path_pattern {
        RequestPathPattern::Exact(pattern) => 
          handle_exact_path(pattern, path, &queries),
        RequestPathPattern::Match(pattern) => 
          handle_regex_path(pattern, request_path, path, &queries),
    }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn params_is_not_none() {
    let result = parse_request_path(
      &RequestPathPattern::Exact(String::from("/projects/:name")),
      "/projects/my-project",
    );
    assert_eq!(result.unwrap().params.get("name").unwrap(), "my-project");
  }

  #[test]
  fn params_is_none() {
    let result = parse_request_path(
      &RequestPathPattern::Exact(String::from("/projects/")),
      "/projects/",
    );
    assert!(result.unwrap().params.is_empty());
  }

  #[test]
  fn request_path_does_not_match() {
    let result = parse_request_path(
      &RequestPathPattern::Exact(String::from("/projects/:name")),
      "/files/",
    );

    assert_eq!(result, None);
  }

  #[test]
  fn test_split_path_and_queries_with_multiple_params() {
    let (path, queries) = 
      split_path_and_queries("/api/users?id=123&name=john");
    assert_eq!(path, "/api/users");
    assert_eq!(queries.get("id").unwrap(), "123");
    assert_eq!(queries.get("name").unwrap(), "john");
  }

  #[test]
  fn test_split_path_and_queries_without_params() {
    let (path, queries) = 
      split_path_and_queries("/api/users");
    assert_eq!(path, "/api/users");
    assert!(queries.is_empty());
  }

  #[test]
  fn test_handle_exact_path_with_params() {
    let queries = HashMap::new();
    let result = 
      handle_exact_path(
        "/users/:id/posts/:post_id", 
        "/users/123/posts/456", 
        &queries
      );
    assert!(result.is_some());
    let path = result.unwrap();
    assert_eq!(path.params.get("id").unwrap(), "123");
    assert_eq!(path.params.get("post_id").unwrap(), "456");
  }

  #[test]
  fn test_handle_exact_path_no_match() {
    let queries = HashMap::new();
    let result = 
      handle_exact_path(
        "/users/:id",
        "/users/123/extra", 
        &queries
      );
    assert!(result.is_none());
  }

  #[test]
  fn test_handle_regex_path_basic() {
    let queries = HashMap::new();
    let result = 
      handle_regex_path(
        r"^/users/(\d+)$",
        "/users/123",
        "/users/123",
        &queries
      );
    assert!(result.is_some());
    let path = result.unwrap();
    assert_eq!(path.matches, vec!["123"]);
  }

  #[test]
  fn test_handle_regex_path_multiple_captures() {
    let queries = HashMap::new();
    let result = handle_regex_path(
      r"^/users/(\d+)/posts/(\w+)$",
      "/users/123/posts/abc",
      "/users/123/posts/abc",
      &queries
    );
    
    assert!(result.is_some());
    let path = result.unwrap();
    assert_eq!(path.matches, vec!["123", "abc"]);
  }

  #[test]
  fn test_handle_regex_path_no_match() {
    let queries = HashMap::new();
    let result = 
      handle_regex_path(
        r"^/users/(\d+)$",
        "/posts/123", 
        "/posts/123",
        &queries
      );
    assert!(result.is_none());
  }
}

pub fn parse_tcp_stream(stream: &mut TcpStream) -> Result<Request, IoError> {
  let mut buf_reader = BufReader::new(stream);
  let mut start_line = String::new();
  buf_reader.read_line(&mut start_line)?;

  let mut start_line_parts = start_line.split_whitespace();
  let method = start_line_parts.next().unwrap().to_uppercase();
  let path = start_line_parts.next().unwrap().to_owned();
  let version = start_line_parts.next().unwrap().to_owned();

  // Read the headers.
  let mut headers = HashMap::new();
  loop {
    let mut line = String::new();
    buf_reader.read_line(&mut line)?;
    if line.trim().is_empty() {
      break;
    }
    if let Some(pos) = line.find(':') {
      let key = line[..pos].trim().to_owned();
      let value = line[pos + 1..].trim().to_owned();
      headers.insert(key, value);
    }
  }

  // Read the body.
  let mut body = String::new();
  if method == "POST" || method == "PUT" {
    let content_length = headers
      .get("Content-Length")
      .and_then(|v| v.parse::<usize>().ok())
      .unwrap_or(0);

    if content_length > 0 {
      let mut buffer = vec![0; content_length];
      buf_reader.read_exact(&mut buffer)?;
      body = String::from_utf8(buffer).unwrap();
    }
  }

  Ok(Request {
    path,
    version,
    method,
    headers,
    body,
    queries: HashMap::new(),
    params: HashMap::new(),
    matches: Vec::new(),
  })
}
