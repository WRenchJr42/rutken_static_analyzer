//! Binary reader with security-critical bounds checking.
//!
//! All reads are little-endian and bounds-checked. Reads past buffer boundaries
//! return errors rather than panicking, making this safe for untrusted input.

use crate::errors::ApkError;

pub struct BinaryReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> BinaryReader<'a> {
    /// Create a new reader over a byte slice.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
        }
    }

    /// Get the current read position in the buffer.
    pub fn position(&self) -> usize {
        self.offset
    }

    /// Create a reader at the specified offset.
    pub fn clone_at(&self, offset: usize) -> Result<Self, ApkError> {
        if offset > self.bytes.len() {
            return Err(ApkError::InvalidFormat("Offset beyond buffer".to_string()));
        }
        Ok(Self {
            bytes: self.bytes,
            offset,
        })
    }

    /// Ensure there are enough bytes remaining to read.
    fn ensure_available(&self, count: usize) -> Result<(), ApkError> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| ApkError::Truncated("read length overflow".to_string()))?;
        if end > self.bytes.len() {
            return Err(ApkError::Truncated("Unexpected end of buffer".to_string()));
        }
        Ok(())
    }

    /// Seek to an absolute offset in the buffer.
    pub fn seek(&mut self, offset: usize) -> Result<(), ApkError> {
        if offset > self.bytes.len() {
            return Err(ApkError::InvalidFormat("Seek beyond end of buffer".to_string()));
        }
        self.offset = offset;
        Ok(())
    }

    /// Read a single byte (bounds-checked).
    pub fn read_u8(&mut self) -> Result<u8, ApkError> {
        self.ensure_available(1)?;
        let value = self.bytes[self.offset];
        self.offset += 1;
        Ok(value)
    }

    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    /// Read a little-endian u16 (2 bytes).
    pub fn read_u16(&mut self) -> Result<u16, ApkError> {
        self.ensure_available(2)?;
        let value = u16::from_le_bytes([self.bytes[self.offset],self.bytes[self.offset + 1]]);
        self.offset +=2;
        Ok(value)
    }
    
    /// Read a little-endian u32 (4 bytes).
    pub fn read_u32(&mut self) -> Result<u32, ApkError> {
        self.ensure_available(4)?;
        let value = u32::from_le_bytes([self.bytes[self.offset], self.bytes[self.offset + 1], self.bytes[self.offset + 2], self.bytes[self.offset + 3]]);
        self.offset += 4;
        Ok(value)
    }

    /// Read a slice of raw bytes.
    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ApkError> {
        self.ensure_available(len)?;
        let slice = &self.bytes[self.offset..self.offset + len];
        self.offset += len;
        Ok(slice)
    }
    
    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], ApkError> {
        self.ensure_available(N)?;
        let mut array = [0u8; N];
        array.copy_from_slice(&self.bytes[self.offset..self.offset+N]);
        self.offset += N;
        Ok(array)
    }

    pub fn read_uleb128(&mut self) -> Result<u32, ApkError> {
        // DEX ULEB128 values are at most 5 bytes (32 bits of payload).
        const MAX_BYTES: u32 = 5;

        let mut result = 0u32;
        let mut shift = 0u32;

        loop {
            let byte = self.read_u8()?;
            if shift < 32 {
                result |= ((byte & 0x7f) as u32) << shift;
            }
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift / 7 >= MAX_BYTES {
                return Err(ApkError::InvalidFormat(
                    "ULEB128 sequence too long".to_string(),
                ));
            }
        }
        Ok(result)
    }
    
    /// Read a signed LEB128 value, as used for `encoded_catch_handler.size`
    /// in DEX `code_item` try/catch data.
    ///
    /// DEX SLEB128 values are at most 5 bytes (32 bits of payload, sign
    /// extended from the last byte read).
    pub fn read_sleb128(&mut self) -> Result<i32, ApkError> {
        const MAX_BYTES: u32 = 5;

        let mut result = 0i64;
        let mut shift = 0u32;
        let mut byte;

        loop {
            byte = self.read_u8()?;
            result |= ((byte & 0x7f) as i64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                break;
            }
            if shift / 7 >= MAX_BYTES {
                return Err(ApkError::InvalidFormat(
                    "SLEB128 sequence too long".to_string(),
                ));
            }
        }

        // Sign-extend if the sign bit of the last byte read is set and
        // there are remaining bits to fill.
        if shift < 64 && (byte & 0x40) != 0 {
            result |= -1i64 << shift;
        }

        Ok(result as i32)
    }

    pub fn read_cstring(&mut self) -> Result<String, ApkError> {
        let start = self.offset;
        while self.read_u8()? != 0 {}
        let end = self.offset - 1;
        Ok(String::from_utf8_lossy(&self.bytes[start..end]).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_uleb128_normal_values_round_trip() {
        // 300 encoded as ULEB128: 0xAC, 0x02
        let mut reader = BinaryReader::new(&[0xAC, 0x02]);
        assert_eq!(reader.read_uleb128().unwrap(), 300);
    }

    #[test]
    fn read_uleb128_rejects_overlong_sequence() {
        // Six continuation bytes: exceeds the 5-byte cap for a 32-bit value.
        let bytes = [0x80, 0x80, 0x80, 0x80, 0x80, 0x00];
        let mut reader = BinaryReader::new(&bytes);
        assert!(reader.read_uleb128().is_err());
    }

    #[test]
    fn read_uleb128_five_bytes_is_still_valid() {
        // Maximum-width encoding of u32::MAX.
        let bytes = [0xff, 0xff, 0xff, 0xff, 0x0f];
        let mut reader = BinaryReader::new(&bytes);
        assert_eq!(reader.read_uleb128().unwrap(), u32::MAX);
    }

    #[test]
    fn ensure_available_does_not_overflow_on_huge_offset() {
        let mut reader = BinaryReader::new(&[1, 2, 3]);
        reader.seek(3).unwrap();
        // Requesting a huge length must not panic via `offset + count` overflow.
        assert!(reader.read_bytes(usize::MAX).is_err());
    }

    #[test]
    fn read_u16_on_too_short_buffer_returns_err() {
        let mut reader = BinaryReader::new(&[0x01]);
        assert!(reader.read_u16().is_err());
    }

    #[test]
    fn read_u16_at_end_of_buffer_returns_err() {
        let mut reader = BinaryReader::new(&[0x01, 0x02]);
        let _ = reader.read_u16(); // reads successfully
        assert!(reader.read_u16().is_err());
    }

    #[test]
    fn read_u32_on_too_short_buffer_returns_err() {
        let mut reader = BinaryReader::new(&[0x01, 0x02, 0x03]);
        assert!(reader.read_u32().is_err());
    }

    #[test]
    fn read_u32_at_end_of_buffer_returns_err() {
        let mut reader = BinaryReader::new(&[0x01, 0x02, 0x03, 0x04]);
        let _ = reader.read_u32(); // reads successfully
        assert!(reader.read_u32().is_err());
    }

    #[test]
    fn read_array_on_too_short_buffer_returns_err() {
        let mut reader = BinaryReader::new(&[0x01, 0x02, 0x03]);
        assert!(reader.read_array::<4>().is_err());
    }

    #[test]
    fn read_array_at_exact_boundary() {
        let mut reader = BinaryReader::new(&[0x01, 0x02, 0x03, 0x04]);
        let array = reader.read_array::<4>().unwrap();
        assert_eq!(array, [0x01, 0x02, 0x03, 0x04]);
        assert!(reader.read_u8().is_err());
    }

    #[test]
    fn read_bytes_on_too_short_buffer_returns_err() {
        let mut reader = BinaryReader::new(&[0x01, 0x02, 0x03]);
        assert!(reader.read_bytes(5).is_err());
    }

    #[test]
    fn read_bytes_with_zero_length_succeeds() {
        let mut reader = BinaryReader::new(&[0x01, 0x02, 0x03]);
        let bytes = reader.read_bytes(0).unwrap();
        assert!(bytes.is_empty());
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn seek_past_end_returns_err() {
        let mut reader = BinaryReader::new(&[0x01, 0x02, 0x03]);
        assert!(reader.seek(4).is_err());
    }

    #[test]
    fn seek_to_exact_end_succeeds() {
        let mut reader = BinaryReader::new(&[0x01, 0x02, 0x03]);
        reader.seek(3).unwrap();
        assert_eq!(reader.position(), 3);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn clone_at_valid_offset() {
        let reader = BinaryReader::new(&[0x01, 0x02, 0x03, 0x04]);
        let cloned = reader.clone_at(2).unwrap();
        assert_eq!(cloned.position(), 2);
    }

    #[test]
    fn clone_at_past_end_returns_err() {
        let reader = BinaryReader::new(&[0x01, 0x02, 0x03]);
        assert!(reader.clone_at(4).is_err());
    }

    #[test]
    fn clone_at_exact_end_succeeds() {
        let reader = BinaryReader::new(&[0x01, 0x02, 0x03]);
        let cloned = reader.clone_at(3).unwrap();
        assert_eq!(cloned.position(), 3);
    }

    #[test]
    fn read_cstring_finds_nul_terminator() {
        let mut reader = BinaryReader::new(b"hello\0world");
        let string = reader.read_cstring().unwrap();
        assert_eq!(string, "hello");
        assert_eq!(reader.position(), 6);
    }

    #[test]
    fn read_cstring_with_no_nul_terminator_returns_err() {
        let mut reader = BinaryReader::new(b"hello");
        assert!(reader.read_cstring().is_err());
    }

    #[test]
    fn read_cstring_with_empty_string() {
        let mut reader = BinaryReader::new(b"\0rest");
        let string = reader.read_cstring().unwrap();
        assert_eq!(string, "");
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn read_uleb128_truncated_with_continuation_bit_set() {
        // A continuation byte at EOF without a following byte.
        let mut reader = BinaryReader::new(&[0x80]); // continuation bit set, but EOF
        assert!(reader.read_uleb128().is_err());
    }

    #[test]
    fn read_uleb128_single_byte_no_continuation() {
        let mut reader = BinaryReader::new(&[0x7f]); // 127, no continuation
        assert_eq!(reader.read_uleb128().unwrap(), 0x7f);
    }

    #[test]
    fn read_uleb128_two_bytes() {
        let mut reader = BinaryReader::new(&[0x80, 0x01]); // 128 encoded as ULEB128
        assert_eq!(reader.read_uleb128().unwrap(), 128);
    }

    #[test]
    fn read_u16_little_endian_round_trip() {
        let mut reader = BinaryReader::new(&[0x34, 0x12]); // 0x1234 in little endian
        assert_eq!(reader.read_u16().unwrap(), 0x1234);
    }

    #[test]
    fn read_sleb128_positive_single_byte() {
        let mut reader = BinaryReader::new(&[0x02]);
        assert_eq!(reader.read_sleb128().unwrap(), 2);
    }

    #[test]
    fn read_sleb128_negative_single_byte() {
        // -1 encoded as SLEB128 is 0x7f.
        let mut reader = BinaryReader::new(&[0x7f]);
        assert_eq!(reader.read_sleb128().unwrap(), -1);
    }

    #[test]
    fn read_sleb128_negative_two_bytes() {
        // -128 encoded as SLEB128 is 0x80, 0x7f.
        let mut reader = BinaryReader::new(&[0x80, 0x7f]);
        assert_eq!(reader.read_sleb128().unwrap(), -128);
    }

    #[test]
    fn read_sleb128_rejects_overlong_sequence() {
        let bytes = [0x80, 0x80, 0x80, 0x80, 0x80, 0x00];
        let mut reader = BinaryReader::new(&bytes);
        assert!(reader.read_sleb128().is_err());
    }

    #[test]
    fn read_sleb128_truncated_with_continuation_bit_set() {
        let mut reader = BinaryReader::new(&[0x80]);
        assert!(reader.read_sleb128().is_err());
    }

    #[test]
    fn read_u32_little_endian_round_trip() {
        let mut reader = BinaryReader::new(&[0x78, 0x56, 0x34, 0x12]); // 0x12345678 in little endian
        assert_eq!(reader.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn position_and_remaining_tracking() {
        let mut reader = BinaryReader::new(&[0x01, 0x02, 0x03, 0x04, 0x05]);
        assert_eq!(reader.position(), 0);
        assert_eq!(reader.remaining(), 5);
        let _ = reader.read_u16().unwrap();
        assert_eq!(reader.position(), 2);
        assert_eq!(reader.remaining(), 3);
        let _ = reader.read_u8().unwrap();
        assert_eq!(reader.position(), 3);
        assert_eq!(reader.remaining(), 2);
    }
}

