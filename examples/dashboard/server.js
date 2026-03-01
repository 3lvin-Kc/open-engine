/**
 * OpenClaw State Engine Dashboard
 * 
 * Simple web dashboard to monitor sessions, goals, and executions.
 * 
 * Run: npm install && npm start
 * Then open http://localhost:8080
 */

const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = process.env.PORT || 8080;
const API_URL = process.env.API_URL || 'http://127.0.0.1:3030';

// Simple JSON-RPC client
async function callApi(method, params = {}) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify({
      jsonrpc: '2.0',
      method,
      params,
      id: Date.now()
    });

    const req = http.request(API_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(data)
      }
    }, (res) => {
      let body = '';
      res.on('data', chunk => body += chunk);
      res.on('end', () => {
        try {
          const result = JSON.parse(body);
          if (result.error) {
            reject(new Error(result.error.message));
          } else {
            resolve(result.result);
          }
        } catch (e) {
          reject(e);
        }
      });
    });

    req.on('error', reject);
    req.write(data);
    req.end();
  });
}

// API proxy endpoint
const server = http.createServer(async (req, res) => {
  // CORS headers
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') {
    res.writeHead(200);
    res.end();
    return;
  }

  // API proxy
  if (req.url === '/api/proxy' && req.method === 'POST') {
    let body = '';
    req.on('data', chunk => body += chunk);
    req.on('end', async () => {
      try {
        const { method, params } = JSON.parse(body);
        const result = await callApi(method, params);
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ success: true, result }));
      } catch (error) {
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ success: false, error: error.message }));
      }
    });
    return;
  }

  // Health check
  if (req.url === '/api/health') {
    try {
      await callApi('get_user', { id: 'test' });
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ status: 'connected', api: API_URL }));
    } catch {
      res.writeHead(503, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ status: 'disconnected', api: API_URL }));
    }
    return;
  }

  // Serve static files
  const filePath = req.url === '/' ? '/index.html' : req.url;
  const fullPath = path.join(__dirname, 'public', filePath);
  
  const ext = path.extname(fullPath);
  const contentType = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.css': 'text/css',
    '.json': 'application/json'
  }[ext] || 'text/plain';

  try {
    const content = fs.readFileSync(fullPath);
    res.writeHead(200, { 'Content-Type': contentType });
    res.end(content);
  } catch {
    res.writeHead(404);
    res.end('Not found');
  }
});

// Ensure public directory exists
const publicDir = path.join(__dirname, 'public');
if (!fs.existsSync(publicDir)) {
  fs.mkdirSync(publicDir, { recursive: true });
}

server.listen(PORT, () => {
  console.log(`╔══════════════════════════════════════════════════════════╗`);
  console.log(`║     OpenClaw State Engine Dashboard                      ║`);
  console.log(`╠══════════════════════════════════════════════════════════╣`);
  console.log(`║  Dashboard: http://localhost:${PORT}                       ║`);
  console.log(`║  API:       ${API_URL}                    ║`);
  console.log(`╚══════════════════════════════════════════════════════════╝`);
  console.log(`\n⚠️  Make sure the state engine is running on ${API_URL}`);
});
