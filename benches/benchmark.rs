#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use std::hint::black_box;
use std::time::Instant;

use http_server::{Frame, PagePool, Request, base64_encode, gzip_compress, sha1};

fn main() {
    println!("Running benchmarks");

    let req_raw = b"GET /api/users/1234?filter=active&sort=desc HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: Benchmark/1.0\r\nAccept: application/json\r\n\r\n";
    let iters = 100_000;
    let t0 = Instant::now();
    for _ in 0..iters {
        let _ = black_box(Request::parse(black_box(req_raw)));
    }
    let d0 = t0.elapsed();
    let rps0 = (f64::from(iters) / d0.as_secs_f64()) as u64;
    println!("HTTP Request Parsing:   {iters} iters in {d0:.2?} ({rps0} req/s)");

    let pool = PagePool::new(16);
    let t1 = Instant::now();
    for _ in 0..iters {
        let mut page = pool.acquire();
        page.as_mut_slice()[0] = 1;
        black_box(&page);
        drop(page);
    }
    let d1 = t1.elapsed();
    let rps1 = (f64::from(iters) / d1.as_secs_f64()) as u64;
    println!("4KB Page Recycle:       {iters} iters in {d1:.2?} ({rps1} ops/s)");

    let sample_payload = b"WebSocket Handshake Nonce Hashing Payload Verification 123456789";
    let t2 = Instant::now();
    for _ in 0..iters {
        let _ = black_box(sha1(black_box(sample_payload)));
    }
    let d2 = t2.elapsed();
    let rps2 = (f64::from(iters) / d2.as_secs_f64()) as u64;
    println!("SHA-1:                  {iters} iters in {d2:.2?} ({rps2} hashes/s)");

    let raw_key = b"dGhlIHNhbXBsZSBub25jZQ==";
    let t3 = Instant::now();
    for _ in 0..iters {
        let _ = black_box(base64_encode(black_box(raw_key)));
    }
    let d3 = t3.elapsed();
    let rps3 = (f64::from(iters) / d3.as_secs_f64()) as u64;
    println!("Base64:                 {iters} iters in {d3:.2?} ({rps3} enc/s)");

    let raw_ws = vec![
        0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58,
    ];
    let t4 = Instant::now();
    for _ in 0..iters {
        let _ = black_box(Frame::decode(black_box(&raw_ws)));
    }
    let d4 = t4.elapsed();
    let rps4 = (f64::from(iters) / d4.as_secs_f64()) as u64;
    println!("WebSocket Frame Decode: {iters} iters in {d4:.2?} ({rps4} frames/s)");

    let raw_text =
        b"Repeated buffer text for high-throughput HTTP gzip compression benchmark testing";
    let gzip_iters = 20_000;
    let t5 = Instant::now();
    for _ in 0..gzip_iters {
        let _ = black_box(gzip_compress(black_box(raw_text)));
    }
    let d5 = t5.elapsed();
    let rps5 = (f64::from(gzip_iters) / d5.as_secs_f64()) as u64;
    println!("Gzip Compression:       {gzip_iters} iters in {d5:.2?} ({rps5} blocks/s)");
}
