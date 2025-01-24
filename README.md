### Note

Avoid using third-party libraries as much as possible.

## queries, headers, body of request

These data are used to match the request data.

## Example

operators: `is`, `is!`, `contains`, `contains!`

```json
{
  "description": "my-project",
  "endpoints": {
    "hello": {
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
