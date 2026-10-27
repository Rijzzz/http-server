use http_server::{Frame, OpCode};

#[test]
fn test_ws_text_frame_unmasked_encode_decode() {
    let payload = b"hello websocket";
    let encoded = Frame::encode_text(payload);

    let (frame, consumed) = Frame::decode(&encoded).expect("frame should decode");
    assert_eq!(consumed, encoded.len());
    assert_eq!(frame.opcode, OpCode::Text);
    assert_eq!(frame.payload, payload);
    assert!(frame.fin);
}

#[test]
fn test_ws_masked_frame_decode() {
    let raw = vec![
        0x81, // fin=1, opcode=1
        0x85, // mask=1, len=5
        0x37, 0xfa, 0x21, 0x3d, // mask key
        0x7f, 0x9f, 0x4d, 0x51, 0x58, // masked payload
    ];

    let (frame, consumed) = Frame::decode(&raw).expect("masked frame should decode");
    assert_eq!(consumed, raw.len());
    assert_eq!(frame.opcode, OpCode::Text);
    assert_eq!(frame.payload, b"Hello");
    assert!(frame.fin);
}

#[test]
fn test_ws_ping_pong_frame() {
    let ping_bytes = Frame::encode_ping(b"heartbeat");
    let (frame, _) = Frame::decode(&ping_bytes).expect("ping should decode");
    assert_eq!(frame.opcode, OpCode::Ping);
    assert_eq!(frame.payload, b"heartbeat");

    let pong_reply = Frame::encode_pong(&frame.payload);
    let (pong_frame, _) = Frame::decode(&pong_reply).expect("pong should decode");
    assert_eq!(pong_frame.opcode, OpCode::Pong);
    assert_eq!(pong_frame.payload, b"heartbeat");
}
