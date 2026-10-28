use http_server::{HeaderMap, Method, Request, Response, StatusCode};

#[test]
fn test_header_map_case_insensitivity() {
    let mut map = HeaderMap::new();
    map.insert("Content-Type", "application/json");
    map.insert("X-Custom-Header", "test-val");

    assert_eq!(map.get("content-type"), Some("application/json"));
    assert_eq!(map.get("CONTENT-TYPE"), Some("application/json"));
    assert_eq!(map.get("Content-Type"), Some("application/json"));
    assert_eq!(map.get("x-custom-header"), Some("test-val"));
    assert_eq!(map.get("missing"), None);
}

#[test]
fn test_request_parse_simple() {
    let raw =
        b"GET /search?q=rust&limit=25 HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: test\r\n\r\n";
    let req = Request::parse(raw).expect("should parse request");

    assert_eq!(req.method, Method::Get);
    assert_eq!(req.path, "/search");
    assert_eq!(req.query.get("q"), Some(&"rust".to_string()));
    assert_eq!(req.query.get("limit"), Some(&"25".to_string()));
    assert_eq!(req.headers.get("host"), Some("localhost:8080"));
    assert_eq!(req.body, Vec::<u8>::new());
    assert!(req.keep_alive);
}

#[test]
fn test_request_parse_with_body() {
    let body = r#"{"name":"ferris"}"#;
    let raw = format!(
        "POST /api/users HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
        body.len(),
        body
    );

    let req = Request::parse(raw.as_bytes()).expect("should parse post request");
    assert_eq!(req.method, Method::Post);
    assert_eq!(req.path, "/api/users");
    assert_eq!(req.body, body.as_bytes());
}

#[test]
fn test_response_wire_format() {
    let res = Response::json(StatusCode::Ok, r#"{"status":"ready"}"#);
    let bytes = res.to_bytes();
    let text = String::from_utf8_lossy(&bytes);

    assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(text.contains("content-type: application/json\r\n"));
    assert!(text.contains("content-length: 18\r\n"));
    assert!(text.ends_with("\r\n\r\n{\"status\":\"ready\"}"));
}

#[test]
fn test_partial_content_response() {
    let mut res = Response::bytes(StatusCode::PartialContent, "text/plain", b"part".to_vec());
    res.header("content-range", "bytes 0-3/10");
    let bytes = res.to_bytes();
    let text = String::from_utf8_lossy(&bytes);

    assert!(text.starts_with("HTTP/1.1 206 Partial Content\r\n"));
    assert!(text.contains("content-range: bytes 0-3/10\r\n"));
    assert!(text.ends_with("\r\n\r\npart"));
}
