#[allow(clippy::cast_possible_truncation)]
const CRC_TABLE: [u32; 256] = {
    let mut table = [0_u32; 256];
    let mut idx = 0;
    while idx < 256 {
        let mut rem = idx as u32;
        let mut bit = 0;
        while bit < 8 {
            if (rem & 1) == 1 {
                rem = (rem >> 1) ^ 0xEDB8_8320;
            } else {
                rem >>= 1;
            }
            bit += 1;
        }
        table[idx] = rem;
        idx += 1;
    }
    table
};

#[must_use]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for &byte in data {
        let idx = ((crc ^ u32::from(byte)) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC_TABLE[idx];
    }
    !crc
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn gzip_compress(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 32);

    // gzip header
    out.extend_from_slice(&[0x1F, 0x8B, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF]);

    // raw blocks
    if data.is_empty() {
        let empty_chunk = [0x01, 0x00, 0x00, 0xFF, 0xFF];
        out.extend_from_slice(&empty_chunk);
    } else {
        let mut pos = 0;
        let len = data.len();

        while pos < len {
            let chunk_len = (len - pos).min(65_535);
            let is_last = (pos + chunk_len) == len;

            let bfinal_btype = u8::from(is_last);
            out.push(bfinal_btype);

            let len_u16 = chunk_len as u16;
            let nlen_u16 = !len_u16;

            out.extend_from_slice(&len_u16.to_le_bytes());
            out.extend_from_slice(&nlen_u16.to_le_bytes());
            out.extend_from_slice(&data[pos..pos + chunk_len]);

            pos += chunk_len;
        }
    }

    // crc32 trailer
    let checksum = crc32(data);
    let isize = (data.len() as u32).to_le_bytes();

    out.extend_from_slice(&checksum.to_le_bytes());
    out.extend_from_slice(&isize);

    out
}
