#![allow(clippy::too_many_lines, clippy::cast_possible_truncation)]

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::{sleep, spawn};
use std::time::Duration;

use http_server::{Frame, OpCode, Request, Response, Router, Server, StatusCode, accept_key};

fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    port
}

#[test]
fn test_live_server_http_and_websocket() {
    let port = find_free_port();
    let addr = format!("127.0.0.1:{port}");

    let mut router = Router::new();
    router.get(
        "/hello",
        |_req: &Request, _params: &HashMap<String, String>| Response::text(StatusCode::Ok, "world"),
    );

    router.post(
        "/echo",
        |req: &Request, _params: &HashMap<String, String>| {
            let body_str = String::from_utf8_lossy(&req.body);
            Response::text(StatusCode::Ok, &body_str)
        },
    );

    router.get(
        "/user/:id",
        |_req: &Request, params: &HashMap<String, String>| {
            let id = params.get("id").cloned().unwrap_or_default();
            Response::json(StatusCode::Ok, &format!("{{\"id\":\"{id}\"}}"))
        },
    );

    let mut server = Server::new(4, router);
    server.set_ws_handler(|mut ws, _stats| {
        while let Ok(Some(frame)) = ws.recv_frame() {
            match frame.opcode {
                OpCode::Text => {
                    let msg = String::from_utf8_lossy(&frame.payload);
                    let reply = format!("echo: {msg}");
                    if ws.send_text(&reply).is_err() {
                        break;
                    }
                }
                OpCode::Close => break,
                OpCode::Ping if ws.send_pong(&frame.payload).is_err() => break,
                _ => {}
            }
        }
    });

    let server_addr = addr.clone();
    spawn(move || {
        let _ = server.listen(&server_addr);
    });

    sleep(Duration::from_millis(150));

    {
        let mut stream = TcpStream::connect(&addr).expect("connect to test server");
        let req = b"GET /hello HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        stream.write_all(req).expect("write request");

        let mut res = Vec::new();
        stream.read_to_end(&mut res).expect("read response");
        let res_str = String::from_utf8_lossy(&res);

        assert!(res_str.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(res_str.ends_with("\r\n\r\nworld"));
    }

    {
        let mut stream = TcpStream::connect(&addr).expect("connect to test server");
        let body = "custom body payload";
        let req = format!(
            "POST /echo HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(req.as_bytes()).expect("write request");

        let mut res = Vec::new();
        stream.read_to_end(&mut res).expect("read response");
        let res_str = String::from_utf8_lossy(&res);

        assert!(res_str.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(res_str.ends_with(&format!("\r\n\r\n{body}")));
    }

    {
        let mut stream = TcpStream::connect(&addr).expect("connect to test server");
        let req = b"GET /user/42 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        stream.write_all(req).expect("write request");

        let mut res = Vec::new();
        stream.read_to_end(&mut res).expect("read response");
        let res_str = String::from_utf8_lossy(&res);

        assert!(res_str.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(res_str.contains("{\"id\":\"42\"}"));
    }

    {
        let mut stream = TcpStream::connect(&addr).expect("connect to test server");
        let req = b"POST /hello HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        stream.write_all(req).expect("write request");

        let mut res = Vec::new();
        stream.read_to_end(&mut res).expect("read response");
        let res_str = String::from_utf8_lossy(&res);

        assert!(res_str.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"));
        assert!(res_str.contains("allow: GET"));
    }

    {
        let mut stream = TcpStream::connect(&addr).expect("connect to test server");
        let client_key = "dGhlIHNhbXBsZSBub25jZQ==";
        let expected_accept = accept_key(client_key);

        let ws_handshake = format!(
            "GET /ws HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {client_key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );

        stream
            .write_all(ws_handshake.as_bytes())
            .expect("send handshake");

        let mut buf = [0_u8; 1024];
        let n = stream.read(&mut buf).expect("read handshake response");
        let handshake_res = String::from_utf8_lossy(&buf[..n]);

        assert!(handshake_res.starts_with("HTTP/1.1 101 Switching Protocols\r\n"));
        assert!(handshake_res.contains(&format!("sec-websocket-accept: {expected_accept}")));

        let msg = b"ping pong";
        let mask = [0x12, 0x34, 0x56, 0x78];
        let mut masked_payload = Vec::with_capacity(msg.len());
        for (i, &b) in msg.iter().enumerate() {
            masked_payload.push(b ^ mask[i % 4]);
        }

        let mut client_frame = vec![
            0x81,                     // fin=1, opcode=1
            0x80 | (msg.len() as u8), // mask=1, len
        ];
        client_frame.extend_from_slice(&mask);
        client_frame.extend_from_slice(&masked_payload);

        stream
            .write_all(&client_frame)
            .expect("send ws client frame");

        let mut echo_buf = [0_u8; 1024];
        let echo_n = stream.read(&mut echo_buf).expect("read ws echo");
        let (server_frame, _) = Frame::decode(&echo_buf[..echo_n]).expect("decode echo frame");

        assert_eq!(server_frame.opcode, OpCode::Text);
        let echo_text = String::from_utf8_lossy(&server_frame.payload);
        assert_eq!(echo_text, "echo: ping pong");
    }
}
