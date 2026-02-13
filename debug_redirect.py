import requests

url_ts = "http://zfruvync.duperab.xyz/live/PE1S9S8U/11EZZUMW/53504.ts"
url_m3u8 = "http://zfruvync.duperab.xyz/live/PE1S9S8U/11EZZUMW/53504.m3u8"
headers = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
}

def check(name, u):
    print(f"Checking {name}...")
    try:
        r = requests.head(u, headers=headers, allow_redirects=False, timeout=5)
        print(f"Status: {r.status_code}")
        if 'Location' in r.headers:
            print(f"Location: {r.headers['Location']}")
        else:
            print("No Location header")
    except Exception as e:
        print(f"Error: {e}")

check("TS", url_ts)
check("M3U8", url_m3u8)
