import akshare as ak
import json
from datetime import datetime, timedelta
from multiprocessing import Pool
import requests
import os
# Create a custom session with headers
session = requests.Session()
session.headers.update({
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36',
    'Accept-Language': 'en-US,en;q=0.9',
    'Accept-Encoding': 'gzip, deflate, br',
    'Connection': 'keep-alive'
})

def fetch_stock_data(symbol, start_date, end_date):
    """Fetch stock data using akshare"""
    try:
        df = ak.stock_zh_a_hist(
            symbol=symbol, period="daily", 
            start_date=start_date, end_date=end_date, adjust=""
        )
        return df
    except Exception as e:
        print(f"Error fetching data for {symbol}: {e}")
        return None

def create_mock_config(stock_data_dict):
    """Create mock server configuration"""
    config = {
        "description": "stock_daily",
        "endpoints": {
            "/OCHL": {
                "when": [
                    {
                        "method": "GET",
                        "request": {
                            "queries": {
                                "stock": {
                                    "operator": "is",
                                    "value": stock_code
                                },
                                "date": {
                                    "operator": "is",
                                    "value": date
                                }
                            }
                        },
                        "response": {
                            "status": 200,
                            "headers": {
                                "content-type": "application/json"
                            },
                            "body": {
                                "open": float(data["开盘"]),
                                "close": float(data["收盘"]),
                                "high": float(data["最高"]),
                                "low": float(data["最低"])
                            }
                        },
                        "delay": 0
                    }
                    for stock_code, dates in stock_data_dict.items()
                    for date, data in dates.items()
                ]
            }
        }
    }
    return config


def process_stock(symbol, start_date, end_date):
    print(f"Processing {symbol}")
    df = fetch_stock_data(symbol, start_date, end_date)
    if df is not None:
        stock_data = {}
        for _, row in df.iterrows():
            date = row['日期'].strftime('%Y%m%d')
            stock_data[date] = row.to_dict()
        return symbol, stock_data
    return symbol, None


def fetch_all_and_save():

    # Fetch all A-share stock symbols
    try:
        spot_df = ak.stock_zh_a_spot_em()
        stock_symbols = spot_df['代码'].tolist()  # Get all stock codes
    except Exception as e:
        print(f"Error fetching stock symbols: {e}")
        return

    # Date range
    start_date = "20250101"
    end_date = "20250125"
    
    # Collect data for all stocks
    stock_data_dict = {}
    total_stocks = len(stock_symbols)
    
    # Filter out already processed symbols
    remaining_symbols = [symbol for symbol in stock_symbols if symbol not in stock_data_dict]
    
    print(f"Start processing {len(remaining_symbols)} stocks")
    with Pool(128) as pool:
        results = pool.starmap(
            process_stock, 
            [(symbol, start_date, end_date) for symbol in remaining_symbols]
        )
        for i, (symbol, data) in enumerate(results):
            if data is not None:
                stock_data_dict[symbol] = data

    print(f"Processed {len(stock_data_dict)} stocks")

    # Create mock configuration
    config = create_mock_config(stock_data_dict)
    
    # Save configuration to file
    try:
        with open('stock_daily_config.json', 'w', encoding='utf-8') as f:
            json.dump(config, f, ensure_ascii=False, indent=2)
    except Exception as e:
        print(f"Error saving configuration to file: {e}")


def post_config(config):
    # Post configuration to server
    try:
        # 25 days data of 5685 stocks, this is a stress test on the POST endpoint of the server
        response = requests.post(
            'http://localhost:8001/projects/stock_daily',
            json=config,
            headers={'Content-Type': 'application/json'}
        )
        print(f"Server response: {response.status_code}")
        print(response.text)
    except Exception as e:
        print(f"Error posting to server: {e}")


def main():
    if not os.path.exists('stock_daily_config.json'):
        fetch_all_and_save()
    else:
        print("stock_daily_config.json already exists")

    config = json.load(open('stock_daily_config.json', 'r', encoding='utf-8'))
    post_config(config)


if __name__ == "__main__":
    main()
