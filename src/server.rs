use std::io::{Read, Result as IoResult, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::{
    Method, PagePool, Request, Response, Router, ServerStats, SharedStats, StatusCode, ThreadPool,
    WebSocket, is_upgrade_request, upgrade_response,
};

pub type WsHandler = Arc<dyn Fn(WebSocket, SharedStats) + Send + Sync>;

pub struct Server {
    router: Arc<Router>,
    pool: ThreadPool,
    page_pool: PagePool,
    stats: SharedStats,
    ws_handler: Option<WsHandler>,
    running: Arc<AtomicBool>,
}

impl Server {
    #[must_use]
    pub fn new(threads: usize, router: Router) -> Self {
        Self {
            router: Arc::new(router),
            pool: ThreadPool::new(threads),
            page_pool: PagePool::new(threads * 4),
            stats: Arc::new(ServerStats::new()),
            ws_handler: None,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn set_ws_handler<F>(&mut self, handler: F)
    where
        F: Fn(WebSocket, SharedStats) + Send + Sync + 'static,
    {
        self.ws_handler = Some(Arc::new(handler));
    }

    #[must_use]
    pub fn stats(&self) -> SharedStats {
        Arc::clone(&self.stats)
    }

    /// # Errors
    /// Returns error if socket binding fails
    pub fn listen(&self, addr: &str) -> IoResult<()> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(false)?;

        while self.running.load(Ordering::Relaxed) {
            let Ok((stream, _)) = listener.accept() else {
                continue;
            };

            let router = Arc::clone(&self.router);
            let page_pool = self.page_pool.clone();
            let stats = Arc::clone(&self.stats);
            let ws_handler = self.ws_handler.clone();

            stats.inc_conn();

            self.pool.execute(move || {
                handle_connection(stream, &router, &page_pool, &stats, ws_handler.as_deref());
                stats.dec_conn();
            });
        }

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

fn handle_connection(
    mut stream: TcpStream,
    router: &Router,
    page_pool: &PagePool,
    stats: &Arc<ServerStats>,
    ws_handler: Option<&(dyn Fn(WebSocket, SharedStats) + Send + Sync)>,
) {
    let timeout = Duration::from_secs(60);
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));

    let mut conn_buf = Vec::with_capacity(4096);

    loop {
        let mut page = page_pool.acquire();
        let n = match stream.read(page.as_mut_slice()) {
            Ok(0) | Err(_) => break,
            Ok(bytes_read) => {
                stats.add_bytes_in(bytes_read);
                bytes_read
            }
        };

        conn_buf.extend_from_slice(&page.as_slice()[..n]);
        drop(page);

        let Some(req) = Request::parse(&conn_buf) else {
            if conn_buf.len() > 65_536 {
                let res = Response::text(StatusCode::PayloadTooLarge, "413 Payload Too Large");
                let bytes = res.to_bytes();
                stats.add_bytes_out(bytes.len());
                let _ = stream.write_all(&bytes);
                break;
            }
            continue;
        };

        stats.inc_req();

        if req.path == "/metrics" && req.method == Method::Get {
            let json = stats.to_json();
            let mut res = Response::json(StatusCode::Ok, &json);
            if req.keep_alive {
                res.header("connection", "keep-alive");
            } else {
                res.header("connection", "close");
            }
            let bytes = res.to_bytes();
            stats.add_bytes_out(bytes.len());
            if stream.write_all(&bytes).is_err() || !req.keep_alive {
                break;
            }
            conn_buf.clear();
            continue;
        }

        if is_upgrade_request(&req.headers)
            && let (Some(key), Some(handler)) = (req.headers.get("sec-websocket-key"), ws_handler)
        {
            let res = upgrade_response(key);
            let bytes = res.to_bytes();
            stats.add_bytes_out(bytes.len());
            if stream.write_all(&bytes).is_ok() {
                let _ = stream.set_read_timeout(None);
                let _ = stream.set_write_timeout(None);
                let ws = WebSocket::new(stream);
                handler(ws, Arc::clone(stats));
            }
            return;
        }

        let keep_alive = req.keep_alive;
        let mut res = router.handle(&req);

        if keep_alive {
            res.header("connection", "keep-alive");
        } else {
            res.header("connection", "close");
        }

        let out_bytes = res.to_bytes();
        stats.add_bytes_out(out_bytes.len());

        if stream.write_all(&out_bytes).is_err() || !keep_alive {
            break;
        }

        conn_buf.clear();
    }
}
