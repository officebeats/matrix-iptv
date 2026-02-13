import requests

domain = "8884152.lbt4.xyz"
doh_url = f"https://dns.google/resolve?name={domain}"

print(f"Querying DoH for {domain}...")
try:
    r = requests.get(doh_url, timeout=5)
    print(r.json())
except Exception as e:
    print(e)
