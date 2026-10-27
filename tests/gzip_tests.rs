use http_server::{crc32, gzip_compress};

#[test]
fn test_crc32_empty_and_text() {
    assert_eq!(crc32(b""), 0);
    assert_eq!(crc32(b"123456789"), 0xcb_f4_39_26);
}

#[test]
fn test_gzip_header_and_trailer() {
    let input = b"Hello, world of compressed HTTP streams!";
    let gz = gzip_compress(input);

    assert_eq!(gz[0], 0x1f);
    assert_eq!(gz[1], 0x8b);
    assert_eq!(gz[2], 0x08);

    let len = gz.len();
    assert!(len >= 18);
    let isize_bytes: [u8; 4] = gz[len - 4..].try_into().expect("isize slice");
    let isize = u32::from_le_bytes(isize_bytes);
    assert_eq!(usize::try_from(isize).expect("isize to usize"), input.len());
}
