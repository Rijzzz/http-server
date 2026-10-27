use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

pub struct ServerStats {
    active_connections: AtomicUsize,
    total_requests: AtomicU64,
    bytes_received: AtomicU64,
    bytes_sent: AtomicU64,
    ws_frames_received: AtomicU64,
    ws_frames_sent: AtomicU64,
    start_time: Instant,
}

impl Default for ServerStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerStats {
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_connections: AtomicUsize::new(0),
            total_requests: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            ws_frames_received: AtomicU64::new(0),
            ws_frames_sent: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn inc_conn(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_conn(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn inc_req(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_bytes_in(&self, n: usize) {
        self.bytes_received.fetch_add(n as u64, Ordering::Relaxed);
    }

    pub fn add_bytes_out(&self, n: usize) {
        self.bytes_sent.fetch_add(n as u64, Ordering::Relaxed);
    }

    pub fn inc_ws_rx(&self) {
        self.ws_frames_received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_ws_tx(&self) {
        self.ws_frames_sent.fetch_add(1, Ordering::Relaxed);
    }

    #[must_use]
    pub fn to_json(&self) -> String {
        let uptime = self.start_time.elapsed().as_secs();
        let conns = self.active_connections.load(Ordering::Relaxed);
        let reqs = self.total_requests.load(Ordering::Relaxed);
        let bin = self.bytes_received.load(Ordering::Relaxed);
        let bout = self.bytes_sent.load(Ordering::Relaxed);
        let ws_in = self.ws_frames_received.load(Ordering::Relaxed);
        let ws_out = self.ws_frames_sent.load(Ordering::Relaxed);

        format!(
            "{{\"uptime_secs\":{uptime},\"active_connections\":{conns},\"total_requests\":{reqs},\"bytes_received\":{bin},\"bytes_sent\":{bout},\"ws_frames_in\":{ws_in},\"ws_frames_out\":{ws_out}}}"
        )
    }
}

pub type SharedStats = Arc<ServerStats>;
