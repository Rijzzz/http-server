const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

const B64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h0 = 0x6745_2301_u32;
    let mut h1 = 0xEFCD_AB89_u32;
    let mut h2 = 0x98BA_DCFE_u32;
    let mut h3 = 0x1032_5476_u32;
    let mut h4 = 0xC3D2_E1F0_u32;

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut buf = Vec::with_capacity(data.len() + 64);
    buf.extend_from_slice(data);
    buf.push(0x80);

    while (buf.len() % 64) != 56 {
        buf.push(0);
    }
    buf.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in buf.as_chunks::<64>().0 {
        let mut w = [0_u32; 80];
        for (idx, slot) in w.iter_mut().take(16).enumerate() {
            let start = idx * 4;
            let end = start + 4;
            let bytes: [u8; 4] = chunk[start..end].try_into().unwrap_or([0; 4]);
            *slot = u32::from_be_bytes(bytes);
        }

        for idx in 16..80 {
            w[idx] = (w[idx - 3] ^ w[idx - 8] ^ w[idx - 14] ^ w[idx - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for (idx, item) in w.iter().enumerate() {
            let (f, k) = match idx {
                0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999_u32),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1_u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC_u32),
                _ => (b ^ c ^ d, 0xCA62_C1D6_u32),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*item);

            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0_u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn base64_encode(src: &[u8]) -> String {
    let mut out = String::with_capacity(src.len().div_ceil(3) * 4);
    let mut idx = 0;
    let len = src.len();

    while idx < len {
        let rem = len - idx;
        if rem >= 3 {
            let b0 = src[idx];
            let b1 = src[idx + 1];
            let b2 = src[idx + 2];
            out.push(B64_CHARS[(b0 >> 2) as usize] as char);
            out.push(B64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
            out.push(B64_CHARS[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
            out.push(B64_CHARS[(b2 & 0x3f) as usize] as char);
            idx += 3;
        } else if rem == 2 {
            let b0 = src[idx];
            let b1 = src[idx + 1];
            out.push(B64_CHARS[(b0 >> 2) as usize] as char);
            out.push(B64_CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
            out.push(B64_CHARS[((b1 & 0x0f) << 2) as usize] as char);
            out.push('=');
            idx += 2;
        } else {
            let b0 = src[idx];
            out.push(B64_CHARS[(b0 >> 2) as usize] as char);
            out.push(B64_CHARS[((b0 & 0x03) << 4) as usize] as char);
            out.push('=');
            out.push('=');
            idx += 1;
        }
    }

    out
}

#[must_use]
pub fn ws_accept_key(client_key: &str) -> String {
    let mut merged = String::with_capacity(client_key.len() + WS_GUID.len());
    merged.push_str(client_key);
    merged.push_str(WS_GUID);
    let digest = sha1(merged.as_bytes());
    base64_encode(&digest)
}
