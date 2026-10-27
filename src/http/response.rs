use crate::{HeaderMap, StatusCode};

#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl Response {
    #[must_use]
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Vec::new(),
        }
    }

    #[must_use]
    pub fn ok() -> Self {
        Self::new(StatusCode::Ok)
    }

    #[must_use]
    pub fn text(status: StatusCode, msg: &str) -> Self {
        let mut res = Self::new(status);
        res.header("content-type", "text/plain; charset=utf-8");
        res.body = msg.as_bytes().to_vec();
        res
    }

    #[must_use]
    pub fn html(status: StatusCode, html_content: &str) -> Self {
        let mut res = Self::new(status);
        res.header("content-type", "text/html; charset=utf-8");
        res.body = html_content.as_bytes().to_vec();
        res
    }

    #[must_use]
    pub fn json(status: StatusCode, json_content: &str) -> Self {
        let mut res = Self::new(status);
        res.header("content-type", "application/json");
        res.body = json_content.as_bytes().to_vec();
        res
    }

    #[must_use]
    pub fn bytes(status: StatusCode, mime: &str, data: Vec<u8>) -> Self {
        let mut res = Self::new(status);
        res.header("content-type", mime);
        res.body = data;
        res
    }

    #[must_use]
    pub fn not_found() -> Self {
        Self::text(StatusCode::NotFound, "404 Not Found")
    }

    #[must_use]
    pub fn method_not_allowed(allowed: &[&str]) -> Self {
        let mut res = Self::text(StatusCode::MethodNotAllowed, "405 Method Not Allowed");
        res.header("allow", &allowed.join(", "));
        res
    }

    #[must_use]
    pub fn redirect(target: &str, status: StatusCode) -> Self {
        let mut res = Self::new(status);
        res.header("location", target);
        res
    }

    pub fn header(&mut self, key: &str, val: &str) -> &mut Self {
        self.headers.insert(key, val);
        self
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let code = self.status.as_u16();
        let reason = self.status.reason();

        let mut out = Vec::with_capacity(256 + self.body.len());
        let status_line = format!("HTTP/1.1 {code} {reason}\r\n");
        out.extend_from_slice(status_line.as_bytes());

        let mut has_content_len = false;
        let mut has_server = false;

        for (k, v) in &self.headers {
            if k.eq_ignore_ascii_case("content-length") {
                has_content_len = true;
            }
            if k.eq_ignore_ascii_case("server") {
                has_server = true;
            }
            let line = format!("{k}: {v}\r\n");
            out.extend_from_slice(line.as_bytes());
        }

        if !has_content_len && self.status != StatusCode::SwitchingProtocols {
            let len_str = format!("content-length: {}\r\n", self.body.len());
            out.extend_from_slice(len_str.as_bytes());
        }

        if !has_server {
            out.extend_from_slice(b"server: RustHttp/1.0\r\n");
        }

        out.extend_from_slice(b"\r\n");
        out.extend_from_slice(&self.body);

        out
    }
}
