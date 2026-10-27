use std::collections::HashMap;
use std::str::from_utf8;

use crate::http::uri::parse_uri;
use crate::{HeaderMap, Method};

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub keep_alive: bool,
}

impl Request {
    #[must_use]
    pub fn parse(buf: &[u8]) -> Option<Self> {
        let header_end = find_header_end(buf)?;
        let header_bytes = &buf[..header_end];
        let header_str = from_utf8(header_bytes).ok()?;

        let mut lines = header_str.split("\r\n");
        let req_line = lines.next()?;
        let mut parts = req_line.split_ascii_whitespace();

        let method_str = parts.next()?;
        let uri_str = parts.next()?;
        let version_str = parts.next().unwrap_or("HTTP/1.1");

        let method = method_str.parse::<Method>().ok()?;
        let (path, query) = parse_uri(uri_str);

        let mut headers = HeaderMap::new();
        for line in lines {
            if line.is_empty() {
                continue;
            }
            if let Some((k, v)) = line.split_once(':') {
                headers.insert(k.trim(), v.trim());
            }
        }

        let body_start = header_end + 4;
        let mut body = Vec::new();

        if let Some(content_len_str) = headers.get("content-length") {
            if let Ok(content_len) = content_len_str.parse::<usize>() {
                let available = buf.len().saturating_sub(body_start);
                let take_len = available.min(content_len);
                body.extend_from_slice(&buf[body_start..body_start + take_len]);
            }
        } else if headers
            .get("transfer-encoding")
            .is_some_and(|te| te.eq_ignore_ascii_case("chunked"))
        {
            body = parse_chunked_body(&buf[body_start..]);
        }

        let keep_alive = if let Some(conn) = headers.get("connection") {
            !conn.eq_ignore_ascii_case("close")
        } else {
            version_str == "HTTP/1.1"
        };

        Some(Self {
            method,
            path,
            query,
            headers,
            body,
            keep_alive,
        })
    }
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    if buf.len() < 4 {
        return None;
    }
    (0..buf.len() - 3).find(|&idx| {
        buf[idx] == b'\r' && buf[idx + 1] == b'\n' && buf[idx + 2] == b'\r' && buf[idx + 3] == b'\n'
    })
}

fn parse_chunked_body(mut src: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    while !src.is_empty() {
        let newline_pos = src
            .windows(2)
            .position(|w| w == b"\r\n")
            .unwrap_or(src.len());

        let line = &src[..newline_pos];
        let line_str = from_utf8(line).unwrap_or("");
        let chunk_size = usize::from_str_radix(line_str.trim(), 16).unwrap_or(0);

        if chunk_size == 0 {
            break;
        }

        let chunk_start = newline_pos + 2;
        let chunk_end = chunk_start + chunk_size;

        if chunk_end <= src.len() {
            body.extend_from_slice(&src[chunk_start..chunk_end]);
            src = if chunk_end + 2 <= src.len() {
                &src[chunk_end + 2..]
            } else {
                &[]
            };
        } else {
            break;
        }
    }
    body
}
