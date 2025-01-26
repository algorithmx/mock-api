import json
import requests
import os

def convert_config(config):
    # Transform the config to use path parameters
    new_config = {
        "description": "stock_daily",
        "endpoints": {}
    }

    # Iterate through the original endpoints and restructure them
    for x in config["endpoints"]["/OCHL"]["when"]:
        date = x["request"]["queries"]["date"]["value"]
        stock_code = x["request"]["queries"]["stock"]["value"]
        endpoint_path = f"/OCHL/{stock_code}/{date}"
        new_config["endpoints"][endpoint_path] = {
            "when": [
                {
                    "method": "GET",
                    # Empty queries
                    "response": x["response"],
                    "delay": 0
                }
            ]
        }

    # Save the transformed config
    with open('stock_daily_config1.json', 'w', encoding='utf-8') as f:
        json.dump(new_config, f, ensure_ascii=False, indent=2)


def post_config(config):
    # Post configuration to server
    try:
        # 25 days data of 5685 stocks, this is a stress test on the POST endpoint of the server
        response = requests.post(
            'http://localhost:8001/projects/stock_daily1',
            json=config,
            headers={'Content-Type': 'application/json'}
        )
        print(f"Server response: {response.status_code}")
        print(response.text)
    except Exception as e:
        print(f"Error posting to server: {e}")


def main():
    if os.path.exists('stock_daily_config.json'):
        if not os.path.exists('stock_daily_config1.json'):
            print("Converting config to use path parameters")
            config = json.load(open('stock_daily_config.json', 'r', encoding='utf-8'))
            convert_config(config)
        else:
            print("stock_daily_config1.json already exists")
    else:
        print("stock_daily_config.json does not exist")

    config = json.load(open('stock_daily_config1.json', 'r', encoding='utf-8'))
    post_config(config)

if __name__ == "__main__":
    main()