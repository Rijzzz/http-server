use std::fmt::Write;

use http_server::{base64_encode, sha1, ws_accept_key};

fn hex_string(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

#[test]
fn test_sha1_vectors() {
    let empty_hash = sha1(b"");
    let hex_empty = hex_string(&empty_hash);
    assert_eq!(hex_empty, "da39a3ee5e6b4b0d3255bfef95601890afd80709");

    let abc_hash = sha1(b"abc");
    let hex_abc = hex_string(&abc_hash);
    assert_eq!(hex_abc, "a9993e364706816aba3e25717850c26c9cd0d89d");
}

#[test]
fn test_base64_vectors() {
    assert_eq!(base64_encode(b""), "");
    assert_eq!(base64_encode(b"f"), "Zg==");
    assert_eq!(base64_encode(b"fo"), "Zm8=");
    assert_eq!(base64_encode(b"foo"), "Zm9v");
    assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
    assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
    assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
}

#[test]
fn test_rfc6455_ws_accept_key() {
    let client_key = "dGhlIHNhbXBsZSBub25jZQ==";
    let accept = ws_accept_key(client_key);
    assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
}
