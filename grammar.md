# Mock API Configuration JSON Grammar Documentation

This document describes the complete structure of the configuration JSON file used to set up a new project endpoint for the mock API server.

---

## 1. Overview

A configuration JSON file for a project endpoint consists of two top‐level keys:

1. **description**  
   A string that describes the project (often the project name) and appears as the value for the project's "description" field.

2. **endpoints**  
   An object (map) where each key represents an API endpoint and the value is its configuration. The endpoint key can either be a full path (e.g., `/api/users`) or a specific identifier (e.g., `/statistics`). Each endpoint configuration is described by one or more conditions specifying how the endpoint should respond.

---

## 2. Root Object

The root object has the following structure:

- **description**: (string, required)  
  Describes the project name or purpose.

- **endpoints**: (object, required)  
  A map of endpoint paths to their corresponding configuration objects.

### Example

```json
{
  "description": "sales",
  "endpoints": {
    "/statistics": { ... },
    "/add/sale": { ... }
  }
}
```


---

## 3. Endpoints Object

Each property in the **endpoints** object represents a single API endpoint and is structured as follows:

- **Key**: The endpoint path (e.g., `/statistics`, `/add/sale`).
- **Value**: An object containing a mandatory **when** key.

### Endpoint Object Structure

```json
{
  "when": [ condition1, condition2, ... ]
}
```

The **when** key holds an array of condition objects. Multiple conditions allow the endpoint to respond differently based on request details.

---

## 4. When Condition Object

Each condition object inside the **when** array must include the following keys:

- **method**: (string, required)  
  Represents the HTTP method (e.g., `"GET"`, `"POST"`, `"PUT"`) this condition applies to.

- **request**: (object, optional)  
  Describes the criteria that an incoming request must meet (such as queries, headers, and body) for this condition to be triggered.

- **response**: (object, required)  
  Specifies the mock response to return when the condition is matched.

- **delay**: (number, required)  
  Specifies the delay (in milliseconds) before sending the response. Typically `0` if no delay is needed.

### When Condition Object Structure

```json
{
  "method": "GET",
  "request": { ... },
  "response": { ... },
  "delay": 0
}
```

---

## 5. Request Matching Object

The optional **request** key within a condition object is used to further qualify the matching criteria of an incoming request. It may include:

- **queries**: (object, optional)  
  A map of query parameter names to their matching rule objects.

- **headers**: (object, optional)  
  A map where each key is a header name and the value is the expected exact header value.

- **body**: (any valid JSON, optional)  
  Represents the expected request body. It is used as an additional matching criterion.

### Request Object Example

```json
{
  "queries": {
    "filter": {
      "operator": "is",
      "value": "active"
    }
  },
  "headers": {
    "content-type": "application/json"
  },
  "body": { "optional": "json body" }
}
```

---

## 6. Query Parameter Matching Object

The **queries** object is a map where each key (a query parameter name) maps to an object which defines how the value should be matched:

- **operator**: (string, required)  
  The match operator. For example:
  - `"is"`: Exact match.
  - `"is!"`: Not equal.
  - `"contains"`: Substring match.
  - `"contains!"`: Does not contain a substring.

- **value**: (string, required)  
  The value to compare against the query parameter.

### Example

```json
{
  "filter": {
    "operator": "is",
    "value": "active"
  }
}
```

---

## 7. Response Object

The **response** key defines the response to be returned when the request meets the condition. It includes:

- **status**: (number, required)  
  HTTP status code (e.g., 200, 400).

- **headers**: (object, required)  
  A map of header names to their corresponding response values.

- **body**: (any valid JSON, optional)  
  The response body which may be an object, array, string, etc.

### Response Object Example

```json
{
  "status": 200,
  "headers": {
    "content-type": "application/json"
  },
  "body": {
    "Q1 total": 100,
    "Q2 total": 200
  }
}
```

---

## 8. Complete Example

Below is an example configuration JSON for a project called `"sales"` which defines two endpoints:

- **Endpoint 1** (`/statistics`):
  - On `GET`, returns mock statistics in JSON format.
  - On `PUT`, accepts new statistics in the request body and returns a success message.

- **Endpoint 2** (`/add/sale`):
  - On `POST`, the provided body is added to the sale record.

```json
{
  "description": "sales",
  "endpoints": {
    "/statistics": {
      "when": [
        {
          "method": "GET",
          "response": {
            "status": 200,
            "headers": {
              "content-type": "application/json"
            },
            "body": {
              "Q1 total": 100,
              "Q2 total": 200
            }
          },
          "delay": 0
        },
        {
          "method": "PUT",
          "request": {
            "body": { "type": "object" }
          },
          "response": {
            "status": 200,
            "headers": {
              "content-type": "text/plain"
            },
            "body": "Statistics updated successfully"
          },
          "delay": 0
        }
      ]
    },
    "/add/sale": {
      "when": [
        {
          "method": "POST",
          "request": {
            "body": { "type": "object" }
          },
          "response": {
            "status": 201,
            "headers": {
              "content-type": "text/plain"
            },
            "body": "Sale record added successfully"
          },
          "delay": 0
        }
      ]
    }
  }
}
```

---

## Summary of Grammar Rules

- **Root Object** must include:
  - `"description"`: string  
  - `"endpoints"`: object mapping endpoint paths to their configurations.

- **Each Endpoint Object** must include:
  - `"when"`: array of condition objects.

- **Each Condition Object** must have:
  - `"method"`: HTTP method as a string.
  - `"response"`: object containing `"status"` (number), `"headers"` (object), and optionally `"body"`.
  - `"delay"`: number specifying the response delay in milliseconds.
  - Optionally, `"request"`: object with `"queries"`, `"headers"`, and `"body"` for request matching.

- **Request -> Queries** is a map where each key is a query parameter and the value is an object with:
  - `"operator"`: string defining the matching operator.
  - `"value"`: string value for comparison.

- **Response Object** requires:
  - `"status"`: number.
  - `"headers"`: object.
  - Optionally, `"body"`: any valid JSON.

This document provides the full specification for creating configuration JSON files to instruct the mock API server on how to match incoming requests and return proper responses.