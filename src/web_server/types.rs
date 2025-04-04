use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct RequestPath {
  pub path: String,
  pub queries: HashMap<String, String>,
  pub params: HashMap<String, String>,
  pub matches: Vec<String>,
}

/// A data structure that represents a request.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Request {
  pub method: Method,
  pub path: String,
  pub version: String,
  pub headers: HashMap<String, String>,
  pub body: String,
  pub queries: HashMap<String, String>,
  pub params: HashMap<String, String>,
  pub matches: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Method {
  Get,
  Post,
  Put,
}

impl Method {
  pub fn to_string(&self) -> String {
    match self {
      Method::Get => String::from("GET"),
      Method::Post => String::from("POST"),
      Method::Put => String::from("PUT"),
    }
  }
}

#[derive(Clone, Debug)]
pub enum RequestPathPattern {
  Exact(String),
  Match(String),
}

pub struct RequestOption {
  pub method: Method,
  pub path: RequestPathPattern,
}

/// A data structure that represents a response.
#[derive(Clone, Debug)]
pub struct Response {
  pub status: u16,
  pub body: String,
  pub headers: HashMap<String, String>,
}

impl Response {
  pub fn html(body: String) -> Response {
    let mut headers = HashMap::new();
    headers.insert(
      String::from("Content-Type"),
      String::from("text/html"),
    );
    
    Response {
      status: 200,
      body,
      headers,
    }
  }
}

/// A data structure that is similar to a [HashMap].
#[derive(serde::Deserialize, Debug)]
pub struct Nested {
  #[serde(flatten)]
  pub values: HashMap<String, NestedValue>,
}

impl Nested {
  pub fn new() -> Self {
    Self {
      values: HashMap::new(),
    }
  }

  pub fn insert(&mut self, key: String, value: NestedValue) {
    self.values.insert(key, value);
  }

  pub fn insert_string(&mut self, key: String, value: String) {
    self.insert(key, NestedValue::Str(value));
  }

  pub fn iter(&self) -> std::collections::hash_map::Iter<String, NestedValue> {
    self.values.iter()
  }

  pub fn len(&self) -> usize {
    self.values.len()
  }
}

/// A value that can be stored in a [Nested].
#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum NestedValue {
  Map(Nested),
  Str(String),
  Bool(bool),
  Int(i32),
  Float(f32),
  Array(Vec<NestedValue>),
}
