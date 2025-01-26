Mock API
===

Modern Web application development process
typically involves frontend and backend 
development. Frontend presents business logic 
to the user via API provided by backend. The 
design of API is crucial to the entire project. 
In the early stage of the development, 
the frontend and backend developers work in parallel 
on a common design goal. In this stage, an API
mock tool is desirable. Such tool is beneficial 
to the frontend developers to allow them to 
immediately test design ideas that involves 
querying on backend API. It also good for backend 
developers to conform to the design goal while 
sprinting to their minimal viable product. 






## [Yunlong]

Hi! This is my first serious Rust project. It mocks the API behavior defined in a configuration file. 

Look at [`prompt/p1.txt`](prompt/p1.txt) for details.

### TODO

I see the full potential in adapting the original `mock-api` project into an AI-enabled API mock tool. 

What I eventually want is:

```plaintext
Human language API requirement specification ==> LLM ==> Configuration Json
```

Now I am almost there with an experimental endpoint `/llm/:name`, which accepts a POST with JSON body:
```json
{
	"api_url":"https://api.deepseek.com/chat/completions",
	"api_model":"deepseek-chat",
  "api_key":"YOUR_API_KEY_HERE",
  "prompt":"(Your API requirement specifications prompt...)"
}
```

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
