use crate::errors::ApkError;

pub struct BinaryReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> BinaryReader<'a> {
    ///Create new reader 
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
        }
    }

    ///Current Position
    pub fn position(&self) -> usize {
        self.offset
    }

    ///Clone at another offset
    pub fn clone_at(&self, offset: usize) -> Result<Self, ApkError> {
        if offset > self.bytes.len() {
            return Err(ApkError::InvalidFormat("Offset beyonf buffer".to_string()));
        }
        Ok(Self {
            bytes: self.bytes,
            offset,
        })
    }

    ///Bounds Check
    fn ensure_available(&self, count: usize) -> Result<(), ApkError> {
        if self.offset + count > self.bytes.len() {
            return Err(ApkError::InvalidFormat("Unexpected end of buffer".to_string()));
        }
        Ok(())
    }

    ///Travel to absolute Offset 
    pub fn seek(&mut self, offset: usize) -> Result<(), ApkError> {
        if offset > self.bytes.len() {
            return Err(ApkError::InvalidFormat("Seek beyond end of buffer".to_string()));
        }
        self.offset = offset;
        Ok(())
    }

    ///Read Little Endian u8
    pub fn read_u8(&mut self) -> Result<u8, ApkError> {
        self.ensure_available(1)?;
        let value = self.bytes[self.offset];
        self.offset += 1;
        Ok(value)
    }

    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    ///Read Little Endian u16
    pub fn read_u16(&mut self) -> Result<u16, ApkError> {
        self.ensure_available(2)?;
        let value = u16::from_le_bytes([self.bytes[self.offset],self.bytes[self.offset + 1]]);
        self.offset +=2;
        Ok(value)
    }
    
    ///Read LittleEndian u32
    pub fn read_u32(&mut self) -> Result<u32, ApkError> {
        self.ensure_available(4)?;
        let value = u32::from_le_bytes([self.bytes[self.offset], self.bytes[self.offset + 1], self.bytes[self.offset + 2], self.bytes[self.offset + 3]]);
        self.offset += 4;
        Ok(value)
    }

    ///Read raw bytes
    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ApkError> {
        self.ensure_available(len)?;
        let slice = &self.bytes[self.offset..self.offset + len];
        self.offset += len;
        Ok(slice)
    }

}


