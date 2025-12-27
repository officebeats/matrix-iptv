const http = require("http");
const https = require("https");
const url = require("url");

const PORT = 8081;

const server = http.createServer((req, res) => {
  // Enable CORS for all requests
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");

  if (req.method === "OPTIONS") {
    res.writeHead(200);
    res.end();
    return;
  }

  // Extract the actual target URL from the request URL path
  // Example: http://localhost:8080/http://provider.com/get...
  const requestUrl = req.url.slice(1); // Remove leading slash

  if (!requestUrl.startsWith("http")) {
    res.writeHead(400, { "Content-Type": "text/plain" });
    res.end("Usage: http://localhost:8080/<TARGET_URL>");
    return;
  }

  console.log(`[Proxy] Forwarding to: ${requestUrl}`);
  const parsedUrl = url.parse(requestUrl);
  console.log(`[Proxy] Target Host: ${parsedUrl.host}`);

  const protocol = parsedUrl.protocol === "https:" ? https : http;

  const headers = { ...req.headers };
  headers.host = parsedUrl.host;
  delete headers.origin;
  delete headers.referer;

  const options = {
    method: req.method,
    headers: headers,
  };

  const proxyReq = protocol.request(requestUrl, options, (proxyRes) => {
    console.log(`[Proxy] Target Status: ${proxyRes.statusCode}`);
    console.log(`[Proxy] Target Headers:`, proxyRes.headers);

    res.writeHead(proxyRes.statusCode, {
      ...proxyRes.headers,
      "Access-Control-Allow-Origin": "*",
    });
    proxyRes.pipe(res);
  });

  proxyReq.on("error", (err) => {
    console.error(`[Proxy Error] ${err.message}`);
    res.writeHead(500, { "Content-Type": "text/plain" });
    res.end(`Proxy Error: ${err.message}`);
  });

  req.pipe(proxyReq);
});

server.listen(PORT, () => {
  console.log(`\n🚀 CORS Proxy running at http://localhost:${PORT}`);
  console.log(`usage: http://localhost:${PORT}/<Target_URL>`);
});
