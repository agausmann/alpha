use bytemuck::{Pod, Zeroable};

pub const COMMON_MAGIC: [u64; 2] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b];
pub const BOOTLOADER_INFO_REQUEST: [u64; 2] = [0xf55038d8e2a1202f, 0x279426fcf5f59740];
pub const TERMINAL_REQUEST: [u64; 2] = [0xc8ac59310c2b0844, 0xa68d0c7265d38878];

/// Byte offset of Request.response from the start of the struct.
///
/// `[u64; 2]; [u64; 2]; u64`
pub const RESPONSE_OFFSET: usize = 40;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Request {
    common_magic: [u64; 2],
    request_id: [u64; 2],
    revision: u64,
    response: u64,
}

impl Request {
    pub fn new(request_id: [u64; 2], revision: u64) -> Self {
        Self {
            common_magic: COMMON_MAGIC,
            request_id,
            revision,
            response: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_offset() {
        let request = Request::new([0, 0], 0);
        let request_location = &request as *const _ as usize;
        let response_location = &request.response as *const _ as usize;
        assert_eq!(response_location - request_location, RESPONSE_OFFSET);
    }
}
