import requests
import sys

url = "http://zfruvync.duperab.xyz/live/PE1S9S8U/11EZZUMW/53504.ts"
headers = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Referer": "http://zfruvync.duperab.xyz/"
}

print(f"Testing URL: {url}")
try:
    with requests.get(url, headers=headers, stream=True, timeout=10) as r:
        print(f"Status Code: {r.status_code}")
        print("Headers:")
        for k, v in r.headers.items():
            print(f"  {k}: {v}")
        
        if r.status_code == 200:
            print("\nReading first 64 bytes...")
            chunk = next(r.iter_content(chunk_size=64))
            print(f"Bytes len: {len(chunk)}")
            if len(chunk) > 0:
                print("SUCCESS: Stream is alive!")
        else:
            print(f"\nFAILURE: Server returned {r.status_code}")

except Exception as e:
    print(f"\nEXCEPTION: {e}")
