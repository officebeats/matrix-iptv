import urllib.request
import json
import time

accounts = [
    {"name": "Strong 8K", "url": "http://pledge78502.cdn-akm.me:80", "user": "7c34d33c9e21", "pass": "037dacb169"},
    {"name": "Trex", "url": "http://line.offcial-trex.pro", "user": "3a6aae52fb", "pass": "39c165888139"},
    {"name": "Strong8k2-PC", "url": "http://zfruvync.duperab.xyz", "user": "PE1S9S8U", "pass": "11EZZUMW"},
    {"name": "Mega OTT 1", "url": "http://line.4smart.in", "user": "45Z88W6", "pass": "Z7PHTX3"}
]

print("=== Matrix IPTV Performance Benchmark (Python) ===")

for acc in accounts:
    print(f"\nProcessing: {acc['name']}")
    
    try:
        # 1. Categories
        start = time.time()
        cat_url = f"{acc['url']}/player_api.php?username={acc['user']}&password={acc['pass']}&action=get_live_categories"
        with urllib.request.urlopen(cat_url, timeout=30) as resp:
            cats = json.loads(resp.read().decode())
            duration = time.time() - start
            print(f"  ✅ Categories: {len(cats)} items in {duration:.2f}s")
            
        # 2. Streams & MSNBC
        print("  🔍 Searching for MSNBC...")
        start = time.time()
        streams_url = f"{acc['url']}/player_api.php?username={acc['user']}&password={acc['pass']}&action=get_live_streams"
        with urllib.request.urlopen(streams_url, timeout=120) as resp:
            streams = json.loads(resp.read().decode())
            duration = time.time() - start
            print(f"  ✅ Streams: {len(streams)} items in {duration:.2f}s")
            
            msnbc = [s for s in streams if "MSNBC" in s.get("name", "")]
            if msnbc:
                print(f"  📍 Found {len(msnbc)} MSNBC streams:")
                for s in msnbc[:3]:
                    stream_id = s.get("stream_id")
                    name = s.get("name")
                    play_url = f"{acc['url']}/live/{acc['user']}/{acc['pass']}/{stream_id}.ts"
                    print(f"    - [{stream_id}] {name}")
                    print(f"      Link: {play_url}")
            else:
                print("  ❌ MSNBC NOT FOUND.")
                
    except Exception as e:
        print(f"  ❌ Error: {e}")

print("\n=== Benchmark Complete ===")
