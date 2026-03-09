import socket
import requests

host = "zfruvync.duperab.xyz"
print(f"Checking host: {host}")

# 1. System DNS
try:
    ip = socket.gethostbyname(host)
    print(f"[System DNS] Resolved to: {ip}")
except Exception as e:
    print(f"[System DNS] Failed: {e}")

# 2. Google DoH
try:
    resp = requests.get(f"https://dns.google/resolve?name={host}", timeout=10)
    data = resp.json()
    if data.get("Status") == 0:
        ips = [ans["data"] for ans in data.get("Answer", []) if ans["type"] == 1]
        print(f"[Google DoH] Resolved to: {ips}")
    else:
        print(f"[Google DoH] Failed with status {data.get('Status')}")
except Exception as e:
    print(f"[Google DoH] Error: {e}")

# 3. Cloudflare DoH
try:
    resp = requests.get(f"https://cloudflare-dns.com/dns-query?name={host}", headers={"Accept": "application/dns-json"}, timeout=10)
    data = resp.json()
    if data.get("Status") == 0:
        ips = [ans["data"] for ans in data.get("Answer", []) if ans["type"] == 1]
        print(f"[Cloudflare DoH] Resolved to: {ips}")
    else:
        print(f"[Cloudflare DoH] Failed with status {data.get('Status')}")
except Exception as e:
    print(f"[Cloudflare DoH] Error: {e}")

# 4. Quad9 DoH
try:
    # Quad9 has a JSON API at https://dns.quad9.net/resolve?name=
    resp = requests.get(f"https://dns.quad9.net/resolve?name={host}", timeout=10)
    data = resp.json()
    if data.get("Status") == 0:
        ips = [ans["data"] for ans in data.get("Answer", []) if ans["type"] == 1]
        print(f"[Quad9 DoH] Resolved to: {ips}")
    else:
         print(f"[Quad9 DoH] Failed with status {data.get('Status')}")
except Exception as e:
    print(f"[Quad9 DoH] Error: {e}")
