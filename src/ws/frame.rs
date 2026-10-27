#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Continuation,
    Text,
    Binary,
    Close,
    Ping,
    Pong,
}

impl OpCode {
    #[must_use]
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x0 => Some(Self::Continuation),
            0x1 => Some(Self::Text),
            0x2 => Some(Self::Binary),
            0x8 => Some(Self::Close),
            0x9 => Some(Self::Ping),
            0xA => Some(Self::Pong),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Continuation => 0x0,
            Self::Text => 0x1,
            Self::Binary => 0x2,
            Self::Close => 0x8,
            Self::Ping => 0x9,
            Self::Pong => 0xA,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub fin: bool,
    pub opcode: OpCode,
    pub payload: Vec<u8>,
}

impl Frame {
    #[must_use]
    pub fn new(fin: bool, opcode: OpCode, payload: Vec<u8>) -> Self {
        Self {
            fin,
            opcode,
            payload,
        }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn decode(buf: &[u8]) -> Option<(Self, usize)> {
        if buf.len() < 2 {
            return None;
        }

        let b0 = buf[0];
        let b1 = buf[1];

        let fin = (b0 & 0x80) != 0;
        let opcode = OpCode::from_u8(b0 & 0x0F)?;

        let is_masked = (b1 & 0x80) != 0;
        let len_code = b1 & 0x7F;

        let mut offset = 2;
        let payload_len: usize = match len_code {
            0..=125 => len_code as usize,
            126 => {
                if buf.len() < offset + 2 {
                    return None;
                }
                let bytes: [u8; 2] = buf[offset..offset + 2].try_into().ok()?;
                offset += 2;
                u16::from_be_bytes(bytes) as usize
            }
            127 => {
                if buf.len() < offset + 8 {
                    return None;
                }
                let bytes: [u8; 8] = buf[offset..offset + 8].try_into().ok()?;
                offset += 8;
                usize::try_from(u64::from_be_bytes(bytes)).ok()?
            }
            _ => return None,
        };

        let mask_key = if is_masked {
            if buf.len() < offset + 4 {
                return None;
            }
            let key: [u8; 4] = buf[offset..offset + 4].try_into().ok()?;
            offset += 4;
            Some(key)
        } else {
            None
        };

        if buf.len() < offset + payload_len {
            return None;
        }

        let mut payload = buf[offset..offset + payload_len].to_vec();
        if let Some(mask) = mask_key {
            for (idx, byte) in payload.iter_mut().enumerate() {
                *byte ^= mask[idx % 4];
            }
        }

        let total_consumed = offset + payload_len;
        Some((
            Self {
                fin,
                opcode,
                payload,
            },
            total_consumed,
        ))
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(10 + self.payload.len());
        let b0 = (if self.fin { 0x80 } else { 0x00 }) | self.opcode.as_u8();
        out.push(b0);

        let len = self.payload.len();
        if len < 126 {
            out.push(len as u8);
        } else if len <= 0xFFFF {
            out.push(126);
            out.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            out.push(127);
            out.extend_from_slice(&(len as u64).to_be_bytes());
        }

        out.extend_from_slice(&self.payload);
        out
    }

    #[must_use]
    pub fn encode_text(data: &[u8]) -> Vec<u8> {
        Self::new(true, OpCode::Text, data.to_vec()).encode()
    }

    #[must_use]
    pub fn encode_binary(data: &[u8]) -> Vec<u8> {
        Self::new(true, OpCode::Binary, data.to_vec()).encode()
    }

    #[must_use]
    pub fn encode_ping(data: &[u8]) -> Vec<u8> {
        Self::new(true, OpCode::Ping, data.to_vec()).encode()
    }

    #[must_use]
    pub fn encode_pong(data: &[u8]) -> Vec<u8> {
        Self::new(true, OpCode::Pong, data.to_vec()).encode()
    }

    #[must_use]
    pub fn encode_close(code: u16, reason: &str) -> Vec<u8> {
        let mut payload = Vec::with_capacity(2 + reason.len());
        payload.extend_from_slice(&code.to_be_bytes());
        payload.extend_from_slice(reason.as_bytes());
        Self::new(true, OpCode::Close, payload).encode()
    }
}
