//! A small, fast HTTP server written from scratch in Rust using only `std`.
//!
//! # For Example
//!
//! ```no_run
//! use http_server::{Request, Response, Router, Server, StatusCode};
//! use std::collections::HashMap;
//!
//! let mut router = Router::new();
//! router.get("/", |_req: &Request, _params: &HashMap<String, String>| {
//!     Response::text(StatusCode::Ok, "hello world")
//! });
//!
//! let server = Server::new(4, router);
//! let _ = server.listen("127.0.0.1:8080");
//! ```

pub mod crypto;
pub mod gzip;
pub mod page;
pub mod pool;
pub mod router;
pub mod server;
pub mod static_files;
pub mod stats;

pub mod http {
    #[path = "headers.rs"]
    pub mod headers;
    #[path = "method.rs"]
    pub mod method;
    #[path = "mime.rs"]
    pub mod mime;
    #[path = "request.rs"]
    pub mod request;
    #[path = "response.rs"]
    pub mod response;
    #[path = "status.rs"]
    pub mod status;
    #[path = "uri.rs"]
    pub mod uri;

    pub use headers::HeaderMap;
    pub use method::Method;
    pub use mime::{detect_mime, is_compressible};
    pub use request::Request;
    pub use response::Response;
    pub use status::StatusCode;
    pub use uri::{decode_uri, parse_uri, sanitize_path};
}

pub mod ws {
    #[path = "frame.rs"]
    pub mod frame;
    #[path = "handshake.rs"]
    pub mod handshake;
    #[path = "stream.rs"]
    pub mod stream;

    pub use frame::{Frame, OpCode};
    pub use handshake::{accept_key, is_upgrade_request, upgrade_response};
    pub use stream::WebSocket;
}

pub use crypto::{base64_encode, sha1, ws_accept_key};
pub use gzip::{crc32, gzip_compress};
pub use http::{
    HeaderMap, Method, Request, Response, StatusCode, detect_mime, is_compressible, sanitize_path,
};
pub use page::{PAGE_SIZE, Page, PagePool};
pub use pool::ThreadPool;
pub use router::Router;
pub use server::{Server, WsHandler};
pub use static_files::StaticFileServer;
pub use stats::{ServerStats, SharedStats};
pub use ws::{Frame, OpCode, WebSocket, accept_key, is_upgrade_request, upgrade_response};
