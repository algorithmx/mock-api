import random
import time
import concurrent.futures
import requests
import akshare as ak


# Configuration
BASE_URL = "http://localhost:8001/projects/stock_daily/OCHL"
STOCK_CODES = ak.stock_zh_a_spot_em()['代码'].tolist()
DATE_RANGE = [f"202501{i:02d}" for i in range(1, 26)]
NUM_REQUESTS = 10000  # Total requests to send
CONCURRENCY = 200  # Number of concurrent requests

def make_request():
    """Make a single request to the endpoint"""
    try:
        stock = random.choice(STOCK_CODES)
        date = random.choice(DATE_RANGE)
        params = {"stock": stock, "date": date}
        print(f"Making request with params: {params}")
        response = requests.get(BASE_URL, params=params)
        return response.status_code == 200
    except Exception as e:
        return False

def run_stress_test():
    """Run the stress test with concurrent requests"""
    start_time = time.time()
    success_count = 0
    failure_count = 0
    report_interval = NUM_REQUESTS // 20
    
    with concurrent.futures.ThreadPoolExecutor(max_workers=CONCURRENCY) as executor:
        futures = [executor.submit(make_request) for _ in range(NUM_REQUESTS)]
        for future in concurrent.futures.as_completed(futures, timeout=60):
            try:
                if future.result():
                    success_count += 1
                else:
                    failure_count += 1
            except Exception as e:
                failure_count += 1
            finally:
                # Ensure future is cleaned up
                del future

    total_time = time.time() - start_time
    print(f"\nStress test completed in {total_time:.2f} seconds")
    print(f"Success: {success_count}, Failures: {failure_count}")
    print(f"Requests per second: {NUM_REQUESTS/total_time:.2f}")

if __name__ == "__main__":
    run_stress_test() 