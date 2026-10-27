use std::collections::HashMap;
use std::path::PathBuf;

#[must_use]
pub fn decode_uri(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut idx = 0;
    while idx < bytes.len() {
        if bytes[idx] == b'%' && idx + 2 < bytes.len() {
            if let Ok(val) = u8::from_str_radix(&src[idx + 1..idx + 3], 16) {
                out.push(val as char);
                idx += 3;
                continue;
            }
        } else if bytes[idx] == b'+' {
            out.push(' ');
            idx += 1;
            continue;
        }
        out.push(bytes[idx] as char);
        idx += 1;
    }
    out
}

#[must_use]
pub fn parse_uri(uri: &str) -> (String, HashMap<String, String>) {
    let mut query = HashMap::new();
    if let Some((path_part, query_part)) = uri.split_once('?') {
        for pair in query_part.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                query.insert(decode_uri(k), decode_uri(v));
            } else if !pair.is_empty() {
                query.insert(decode_uri(pair), String::new());
            }
        }
        (decode_uri(path_part), query)
    } else {
        (decode_uri(uri), query)
    }
}

#[must_use]
pub fn sanitize_path(raw: &str) -> PathBuf {
    let mut safe = PathBuf::new();
    for seg in raw.split('/') {
        if seg.is_empty() || seg == "." || seg == ".." {
            continue;
        }
        safe.push(seg);
    }
    safe
}
