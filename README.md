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


This project aims for a configurable API 
mock tool. The user can create an endpoint 
and define its behavior by POSTing 
a configuration json to the API mock server. 
Afterwards, the endpoint responses to various 
queries from any user based on such configuration. 

The API mock server is optimized to be  
responsive and it can survive modest amount 
of concurrent incoming requests.

This is my first serious Rust project. Please open issues on whatever reproducible problems you have ever 
encountered. 

---

## Setup and launch

### Compile and test

The project is ready to be compiled by `cargo build`. 
However, additional preparation on environment 
is necessary, namely:
  - Server port: the env var `MOCK_SERVER_PORT` defines te server port.
    If it is not privided, the server listens on port 53500 on the localhost.
  - Data root folder: the env var `MOCK_SERVER_DB_ROOT` defines the root folder 
    where the server persists the data from the user. Such folder 
    must exist and must contain a writable subfolder with name
    `projects`.

To compile and test the code, run the following inside the folder where `Cargo.toml` is located:
```bash
$ mkdir -p projects
$ export MOCK_SERVER_PORT=8001 && export RUST_BACKTRACE=1 && export MOCK_SERVER_DB_ROOT=`pwd` && cargo build && cargo test
```

### Launch

To run the server, just modify the last word in the above command, 
from `test` to `run`:
```bash
$ mkdir -p projects
$ export MOCK_SERVER_PORT=8001 && export RUST_BACKTRACE=1 && export MOCK_SERVER_DB_ROOT=`pwd` && cargo build && cargo run
```


---

## Using the API mock server

The following instructions assumes that the 
API mock server is successfully launched 
at port 8001 on localhost. 

To start off, you can check out the API documentation 
at endpoint  
`http://localhost:8001/api-doc`, or directly open the file [api-doc.html](src/api-doc.html) for help.

The API mock server provides the following endpoints:

1. **GET /projects/:name** - Retrieve a project's configuration
2. **POST /projects/:name** - Create a new project configuration
3. **PUT /projects/:name** - Update an existing project configuration
4. **GET /projects/:project_name/:path** - Mock an API endpoint based on project configuration
5. **POST /llm/:name** - Generate a project configuration using a Language Model (LLM)
6. **GET /api-doc** - Returns the API documentation page

---

## Main contributions from Yunlong

1. Implemented full functionalities

2. Added `schema.rs` file as a format constraint for the configuration json

3. Added LLM support, so that the user can create mock API
from natural language.

4. Refactored codes

5. Performance optimization

6. Rewrote the regex for the endpoint with query string and redesigned the logic behind the `mock_request`

7. Remove the code for thread pool and use `tikio` to handle concurrent requests.

8. Added `/api-doc` page

9. Added tests



## TODO

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
