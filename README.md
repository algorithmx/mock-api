Note
===

## [Yunlong]

Hi! This is my first serious Rust project. It mocks the API behavior defined in a configuration file. 

Look at [`prompts/p1.txt`](prompts/p1.txt) for details.

### Launch

To test:
```bash
$ mkdir projects
$ export MOCK_SERVER_PORT=8001 && export RUST_BACKTRACE=1 && export MOCK_SERVER_DB_ROOT=`pwd` && cargo build && cargo test
```

To run:
```bash
$ mkdir -p projects
$ export MOCK_SERVER_PORT=8001 && export RUST_BACKTRACE=1 && export MOCK_SERVER_DB_ROOT=`pwd` && cargo build && cargo run
```

### Main contributions

1. Implemented full functionalities

2. Added `schema.rs` file as a format constraint for the configuration json

3. Added tests

4. Refactored codes

5. Rewrote the regex for the endpoint with query string and redesign the logic behind the `mock_request`

6. Added `/api-doc` page

---

## [Original Authors]


Avoid using third-party libraries as much as possible.

## queries, headers, body of request

These data are used to match the request data.

## Example

operators: `is`, `is!`, `contains`, `contains!`

```json
{
  "description": "my-project",
  "endpoints": {
    "/hello": {
      "when": [
        {
          "method": "GET",
          "request": {
            "queries": {
              "name": {
                "operator": "is!",
                "value": "foo"
              }
            },
            "headers": {
              "token": "go"
            }
          },
          "response": {
            "status": 200,
            "body": {},
            "headers": {}
          },
          "delay": 400
        },
        {
          "method": "POST",
          "request": {
            "headers": {
              "content-type": "xxx"
            },
            "body": {
              "name": "foo"
            }
          },
          "response": {
            "status": 200,
            "body": {},
            "headers": {}
          }
        }
      ]
    }
  }
}
```
