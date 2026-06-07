use std::io::{Result, Error, ErrorKind};
use tokio_uring::fs::File;
use crate::io::AlignedBuffer;
use crc32fast::Hasher;

pub const WAL_MAGIC: u8 = 0x52; 
pub const HEADER_SIZE: usize = 14; 
pub const FOOTER_SIZE: usize = 4;  

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpType {
    InsertNode = 1,
    UpdateEdges = 2,
}

pub struct WalRecord {
    pub seq: u64,
    pub op: OpType,
    pub payload: Vec<u8>,
}

pub struct WalWriter {
    file: File,
    offset: u64,
    next_seq: u64,
}

impl WalWriter {
    pub async fn append(&mut self, op: OpType, payload: &[u8]) -> Result<u64> {
        let seq = self.next_seq;
        let record_size = HEADER_SIZE + payload.len() + FOOTER_SIZE;
        let mut buffer = AlignedBuffer::new(record_size);
        
        unsafe {
            let slice = std::slice::from_raw_parts_mut(buffer.stable_mut_ptr(), record_size);
            slice[0] = WAL_MAGIC;
            slice[1..9].copy_from_slice(&seq.to_be_bytes());
            slice[9] = op as u8;
            slice[10..14].copy_from_slice(&(payload.len() as u32).to_be_bytes());
            slice[14..14 + payload.len()].copy_from_slice(payload);
            let mut hasher = Hasher::new();
            hasher.update(&slice[..14 + payload.len()]);
            let crc = hasher.finalize();
            slice[14 + payload.len()..record_size].copy_from_slice(&crc.to_be_bytes());
        }

        let (res, _) = self.file.write_at(buffer, self.offset).await;
        let bytes_written = res?;
        self.offset += bytes_written as u64;
        self.next_seq += 1;
        Ok(seq)
    }
}
