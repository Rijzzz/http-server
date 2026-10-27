use std::collections::HashMap;
use std::env::args;
use std::io::Result as IoResult;

use http_server::{OpCode, Request, Response, Router, Server, StatusCode, gzip_compress};

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Server Dashboard</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: #0d1117;
      color: #c9d1d9;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      line-height: 1.5;
      padding: 24px;
    }
    .container {
      max-width: 960px;
      margin: 0 auto;
    }
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-bottom: 16px;
      border-bottom: 1px solid #21262d;
      margin-bottom: 24px;
      flex-wrap: wrap;
      gap: 12px;
    }
    .brand h1 {
      font-size: 20px;
      font-weight: 600;
      color: #f0f6fc;
    }
    .status-text {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      font-size: 13px;
      color: #8b949e;
    }
    .dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: #8b949e;
      display: inline-block;
    }
    .dot.online { background: #3fb950; }
    .dot.offline { background: #f85149; }
    
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 16px;
      margin-bottom: 24px;
    }
    .metric-card {
      background: #161b22;
      border: 1px solid #30363d;
      border-radius: 6px;
      padding: 16px;
    }
    .metric-card .title {
      font-size: 13px;
      color: #8b949e;
      margin-bottom: 4px;
    }
    .metric-card .val {
      font-size: 26px;
      font-weight: 600;
      color: #f0f6fc;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    }
    .metric-card .sub {
      font-size: 12px;
      color: #8b949e;
      margin-top: 4px;
    }
    
    .panel {
      background: #161b22;
      border: 1px solid #30363d;
      border-radius: 6px;
      margin-bottom: 24px;
    }
    .panel-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 14px 16px;
      border-bottom: 1px solid #21262d;
    }
    .panel-header h2 {
      font-size: 14px;
      font-weight: 600;
      color: #f0f6fc;
    }
    .panel-header .controls {
      display: flex;
      gap: 8px;
    }
    .panel-body {
      padding: 16px;
    }
    
    button {
      background: #21262d;
      color: #c9d1d9;
      border: 1px solid #30363d;
      padding: 6px 12px;
      border-radius: 6px;
      font-size: 13px;
      cursor: pointer;
    }
    button:hover {
      background: #30363d;
      color: #f0f6fc;
    }
    button.primary {
      background: #238636;
      border-color: #2ea043;
      color: #ffffff;
    }
    button.primary:hover {
      background: #2ea043;
    }
    
    .terminal {
      background: #090d13;
      border: 1px solid #21262d;
      border-radius: 4px;
      height: 240px;
      overflow-y: auto;
      padding: 12px;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 12px;
    }
    .log-row {
      display: flex;
      gap: 8px;
      margin-bottom: 4px;
      line-height: 1.4;
      word-break: break-all;
    }
    .log-time {
      color: #484f58;
      flex-shrink: 0;
    }
    .log-tag {
      font-size: 11px;
      flex-shrink: 0;
    }
    .tag-sys { color: #8b949e; }
    .tag-in { color: #58a6ff; }
    .tag-out { color: #3fb950; }
    .tag-err { color: #f85149; }
    .log-text { color: #c9d1d9; }
    
    .input-row {
      display: flex;
      gap: 8px;
      margin-top: 12px;
    }
    .input-row input {
      flex: 1;
      background: #090d13;
      border: 1px solid #30363d;
      color: #f0f6fc;
      padding: 8px 12px;
      border-radius: 6px;
      font-size: 13px;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      outline: none;
    }
    .input-row input:focus {
      border-color: #58a6ff;
    }
    
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 13px;
    }
    th, td {
      text-align: left;
      padding: 10px 16px;
      border-bottom: 1px solid #21262d;
    }
    th {
      color: #8b949e;
      font-weight: 500;
      font-size: 12px;
    }
    td code {
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      color: #c9d1d9;
    }
    .method-get { color: #58a6ff; font-weight: 600; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
    .method-post { color: #3fb950; font-weight: 600; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
    .method-ws { color: #d29922; font-weight: 600; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
    a.test-link {
      color: #58a6ff;
      text-decoration: none;
    }
    a.test-link:hover {
      text-decoration: underline;
    }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <div class="brand">
        <h1>Server Dashboard</h1>
      </div>
      <div id="wsStatus" class="status-text">
        <span class="dot" id="wsDot"></span>
        <span id="wsLabel">Connecting...</span>
      </div>
    </header>

    <div class="grid">
      <div class="metric-card">
        <div class="title">Active Clients</div>
        <div class="val" id="metricConns">0</div>
        <div class="sub">Open connections</div>
      </div>
      <div class="metric-card">
        <div class="title">Total Requests</div>
        <div class="val" id="metricReqs">0</div>
        <div class="sub">Requests handled</div>
      </div>
      <div class="metric-card">
        <div class="title">Network Traffic</div>
        <div class="val" id="metricBytes">0 B</div>
        <div class="sub" id="metricBytesDetail">0 B in / 0 B out</div>
      </div>
      <div class="metric-card">
        <div class="title">WebSocket Messages</div>
        <div class="val" id="metricFrames">0</div>
        <div class="sub" id="metricFramesDetail">0 in / 0 out</div>
      </div>
    </div>

    <div class="panel">
      <div class="panel-header">
        <h2>Live WebSocket Chat</h2>
        <div class="controls">
          <button id="btnClearLog" type="button">Clear</button>
          <button id="btnReconnect" type="button">Reconnect</button>
        </div>
      </div>
      <div class="panel-body">
        <div id="terminal" class="terminal" role="log" aria-live="polite"></div>
        <form id="wsForm" class="input-row">
          <input type="text" id="wsInput" placeholder="Type a message and hit Enter..." value="Hello from browser!">
          <button type="submit" class="primary" id="btnSend">Send</button>
        </form>
      </div>
    </div>

    <div class="panel">
      <div class="panel-header">
        <h2>Server Routes</h2>
        <div class="controls">
          <button id="btnRefreshStats" type="button">Refresh Metrics</button>
        </div>
      </div>
      <div class="panel-body" style="padding: 0;">
        <table>
          <thead>
            <tr>
              <th>Method</th>
              <th>Route</th>
              <th>Description</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><span class="method-get">GET</span></td>
              <td><code>/api/health</code></td>
              <td>Server status check</td>
              <td><a class="test-link btn-try" href="/api/health" data-url="/api/health">Test in console</a></td>
            </tr>
            <tr>
              <td><span class="method-get">GET</span></td>
              <td><code>/api/users/:id</code></td>
              <td>User data with URL parameter</td>
              <td><a class="test-link btn-try" href="/api/users/42" data-url="/api/users/42">Test /42</a></td>
            </tr>
            <tr>
              <td><span class="method-get">GET</span></td>
              <td><code>/compress</code></td>
              <td>Gzip compressed sample response</td>
              <td><a class="test-link btn-try" href="/compress" data-url="/compress">Test in console</a></td>
            </tr>
            <tr>
              <td><span class="method-get">GET</span></td>
              <td><code>/metrics</code></td>
              <td>Live server statistics JSON</td>
              <td><a class="test-link btn-try" href="/metrics" data-url="/metrics">Test in console</a></td>
            </tr>
            <tr>
              <td><span class="method-post">POST</span></td>
              <td><code>/api/echo</code></td>
              <td>Echoes request body back</td>
              <td><span style="color:#8b949e">Use curl</span></td>
            </tr>
            <tr>
              <td><span class="method-ws">WS</span></td>
              <td><code>/ws</code></td>
              <td>Echo WebSocket connection</td>
              <td><span style="color:#3fb950">Active above</span></td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>

  <script>
    (function() {
      let ws = null;
      const term = document.getElementById('terminal');
      const wsDot = document.getElementById('wsDot');
      const wsLabel = document.getElementById('wsLabel');
      const wsInput = document.getElementById('wsInput');
      const wsForm = document.getElementById('wsForm');
      const btnClear = document.getElementById('btnClearLog');
      const btnReconnect = document.getElementById('btnReconnect');
      const btnRefresh = document.getElementById('btnRefreshStats');

      function pad(n) { return n < 10 ? '0' + n : n; }
      function getTimestamp() {
        const d = new Date();
        return pad(d.getHours()) + ':' + pad(d.getMinutes()) + ':' + pad(d.getSeconds());
      }

      function appendLog(type, text) {
        const row = document.createElement('div');
        row.className = 'log-row';

        const timeSpan = document.createElement('span');
        timeSpan.className = 'log-time';
        timeSpan.textContent = '[' + getTimestamp() + ']';

        const tagSpan = document.createElement('span');
        tagSpan.className = 'log-tag tag-' + type.toLowerCase();
        tagSpan.textContent = '[' + type.toLowerCase() + ']';

        const textSpan = document.createElement('span');
        textSpan.className = 'log-text';
        textSpan.textContent = text;

        row.appendChild(timeSpan);
        row.appendChild(tagSpan);
        row.appendChild(textSpan);

        term.appendChild(row);
        term.scrollTop = term.scrollHeight;
      }

      function connectWs() {
        if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) {
          return;
        }

        const proto = location.protocol === 'https:' ? 'wss://' : 'ws://';
        const url = proto + location.host + '/ws';
        appendLog('sys', 'Connecting to ' + url);
        wsDot.className = 'dot';
        wsLabel.textContent = 'Connecting...';

        try {
          ws = new WebSocket(url);
        } catch (err) {
          appendLog('err', 'Could not open WebSocket: ' + err.message);
          return;
        }

        ws.onopen = function() {
          wsDot.className = 'dot online';
          wsLabel.textContent = 'Connected';
          appendLog('sys', 'Connected to server');
          updateMetrics();
        };

        ws.onmessage = function(e) {
          appendLog('in', e.data);
          updateMetrics();
        };

        ws.onerror = function() {
          appendLog('err', 'Network error');
        };

        ws.onclose = function() {
          wsDot.className = 'dot offline';
          wsLabel.textContent = 'Disconnected';
          appendLog('sys', 'Connection closed');
          updateMetrics();
          setTimeout(connectWs, 2000);
        };
      }

      wsForm.addEventListener('submit', function(e) {
        e.preventDefault();
        const msg = wsInput.value.trim();
        if (!msg) return;

        if (ws && ws.readyState === WebSocket.OPEN) {
          ws.send(msg);
          appendLog('out', msg);
          wsInput.value = '';
          setTimeout(updateMetrics, 50);
        } else {
          appendLog('err', 'WebSocket is not connected');
        }
      });

      btnClear.addEventListener('click', function() {
        term.textContent = '';
        appendLog('sys', 'Log cleared');
      });

      btnReconnect.addEventListener('click', function() {
        if (ws) {
          ws.close();
        }
        connectWs();
      });

      btnRefresh.addEventListener('click', function() {
        updateMetrics();
      });

      document.querySelectorAll('.btn-try').forEach(function(el) {
        el.addEventListener('click', function(e) {
          e.preventDefault();
          const url = el.getAttribute('data-url');
          appendLog('sys', 'GET ' + url);
          fetch(url)
            .then(function(res) {
              return res.text().then(function(txt) {
                appendLog('in', '[' + res.status + '] ' + txt);
                updateMetrics();
              });
            })
            .catch(function(err) {
              appendLog('err', 'Request failed: ' + err.message);
            });
        });
      });

      function formatNum(n) {
        return (n || 0).toLocaleString();
      }

      function formatBytes(b) {
        const bytes = b || 0;
        if (bytes < 1024) return bytes + ' B';
        if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
        return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
      }

      function updateMetrics() {
        fetch('/metrics')
          .then(function(res) { return res.json(); })
          .then(function(data) {
            const bin = data.bytes_received !== undefined ? data.bytes_received : (data.bytes_read || 0);
            const bout = data.bytes_sent !== undefined ? data.bytes_sent : (data.bytes_written || 0);
            const win = data.ws_frames_in !== undefined ? data.ws_frames_in : (data.ws_frames_rx || 0);
            const wout = data.ws_frames_out !== undefined ? data.ws_frames_out : (data.ws_frames_tx || 0);

            document.getElementById('metricConns').textContent = formatNum(data.active_connections || 0);
            document.getElementById('metricReqs').textContent = formatNum(data.total_requests || 0);

            const totalBytes = bin + bout;
            document.getElementById('metricBytes').textContent = formatBytes(totalBytes);
            document.getElementById('metricBytesDetail').textContent =
              formatBytes(bin) + ' in / ' + formatBytes(bout) + ' out';

            const totalFrames = win + wout;
            document.getElementById('metricFrames').textContent = formatNum(totalFrames);
            document.getElementById('metricFramesDetail').textContent =
              formatNum(win) + ' in / ' + formatNum(wout) + ' out';
          })
          .catch(function(err) {
            // ignore poll error
          });
      }

      document.addEventListener('visibilitychange', function() {
        if (!document.hidden) {
          updateMetrics();
        }
      });
      window.addEventListener('focus', function() {
        updateMetrics();
      });

      connectWs();
      updateMetrics();
      setInterval(updateMetrics, 1000);
    })();
  </script>
</body>
</html>
"#;

fn build_router() -> Router {
    let mut router = Router::new();

    router.get("/", |_req: &Request, _params: &HashMap<String, String>| {
        Response::html(StatusCode::Ok, DASHBOARD_HTML)
    });

    router.get(
        "/api/health",
        |_req: &Request, _params: &HashMap<String, String>| {
            Response::json(StatusCode::Ok, r#"{"status":"healthy","uptime":"online"}"#)
        },
    );

    router.post(
        "/api/echo",
        |req: &Request, _params: &HashMap<String, String>| {
            let body_str = String::from_utf8_lossy(&req.body);
            Response::text(StatusCode::Ok, &body_str)
        },
    );

    router.get(
        "/api/users/:id",
        |_req: &Request, params: &HashMap<String, String>| {
            let id = params.get("id").cloned().unwrap_or_default();
            Response::json(
                StatusCode::Ok,
                &format!("{{\"user_id\":\"{id}\",\"role\":\"member\"}}"),
            )
        },
    );

    router.get(
        "/compress",
        |_req: &Request, _params: &HashMap<String, String>| {
            let raw_text = "Gzip compressed sample payload ".repeat(20);
            let compressed = gzip_compress(raw_text.as_bytes());
            let mut res = Response::bytes(StatusCode::Ok, "text/plain; charset=utf-8", compressed);
            res.header("content-encoding", "gzip");
            res
        },
    );

    router
}

fn main() -> IoResult<()> {
    let mut port = 8080_u16;
    let mut threads = 8_usize;

    let mut cli_args = args().skip(1);
    while let Some(arg) = cli_args.next() {
        if arg == "--help" || arg == "-h" {
            println!("http-server\n");
            println!("Usage:");
            println!("  http-server [options]\n");
            println!("Options:");
            println!("  --port <num>       Port to bind (default 8080)");
            println!("  --threads <num>    Worker pool size (default 8)");
            println!("  -h, --help         Print help");
            return Ok(());
        } else if arg == "--port"
            && let Some(val) = cli_args.next()
            && let Ok(p) = val.parse::<u16>()
        {
            port = p;
        } else if arg == "--threads"
            && let Some(val) = cli_args.next()
            && let Ok(t) = val.parse::<usize>()
        {
            threads = t;
        }
    }

    let router = build_router();
    let mut server = Server::new(threads, router);

    server.set_ws_handler(|mut ws, stats| {
        while let Ok(Some(frame)) = ws.recv_frame() {
            stats.inc_ws_rx();
            stats.add_bytes_in(frame.payload.len() + 6);
            match frame.opcode {
                OpCode::Text => {
                    let text = String::from_utf8_lossy(&frame.payload);
                    let reply = if text.trim() == "stats" {
                        format!("live stats: {}", stats.to_json())
                    } else {
                        format!("server echo: {text}")
                    };
                    if ws.send_text(&reply).is_ok() {
                        stats.inc_ws_tx();
                        stats.add_bytes_out(reply.len() + 2);
                    } else {
                        break;
                    }
                }
                OpCode::Binary => {
                    if ws.send_binary(&frame.payload).is_ok() {
                        stats.inc_ws_tx();
                        stats.add_bytes_out(frame.payload.len() + 2);
                    } else {
                        break;
                    }
                }
                OpCode::Ping => {
                    if ws.send_pong(&frame.payload).is_ok() {
                        stats.inc_ws_tx();
                        stats.add_bytes_out(frame.payload.len() + 2);
                    } else {
                        break;
                    }
                }
                OpCode::Close => break,
                OpCode::Pong | OpCode::Continuation => {}
            }
        }
    });

    let addr = format!("127.0.0.1:{port}");
    println!("Listening on http://{addr} ({threads} threads)");
    println!("Endpoints:");
    println!("  GET  /");
    println!("  GET  /api/health");
    println!("  POST /api/echo");
    println!("  GET  /api/users/:id");
    println!("  GET  /compress");
    println!("  GET  /metrics");
    println!("  WS   /ws");

    server.listen(&addr)
}
