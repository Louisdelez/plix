//! Response buffer management for host-to-mod communication
//!
//! The host provides a fixed-size buffer for returning data to mods.
//! Mods call plix_response_ptr() and plix_response_len() to access results.

/// Maximum size of the response buffer (8 KB)
pub const RESPONSE_BUFFER_SIZE: usize = 8 * 1024;

/// Host-owned buffer for returning data to WASM mods
#[derive(Debug)]
pub struct ResponseBuffer {
    /// The actual buffer data
    data: [u8; RESPONSE_BUFFER_SIZE],
    /// Current length of valid data in the buffer
    len: usize,
}

impl Default for ResponseBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseBuffer {
    /// Create a new empty response buffer
    pub fn new() -> Self {
        Self {
            data: [0u8; RESPONSE_BUFFER_SIZE],
            len: 0,
        }
    }

    /// Clear the buffer (reset length to 0)
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Write data to the buffer
    ///
    /// Returns the number of bytes written, which may be less than
    /// the input length if the buffer is full.
    pub fn write(&mut self, data: &[u8]) -> usize {
        let available = RESPONSE_BUFFER_SIZE - self.len;
        let to_write = data.len().min(available);

        if to_write > 0 {
            self.data[self.len..self.len + to_write].copy_from_slice(&data[..to_write]);
            self.len += to_write;
        }

        to_write
    }

    /// Write all data to the buffer, failing if it doesn't fit
    pub fn write_all(&mut self, data: &[u8]) -> Result<(), BufferFullError> {
        if data.len() > RESPONSE_BUFFER_SIZE - self.len {
            return Err(BufferFullError {
                needed: data.len(),
                available: RESPONSE_BUFFER_SIZE - self.len,
            });
        }

        self.data[self.len..self.len + data.len()].copy_from_slice(data);
        self.len += data.len();
        Ok(())
    }

    /// Get the current length of valid data
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get a reference to the valid data in the buffer
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// Get a pointer to the start of the buffer
    ///
    /// This is used by the host to provide the response_ptr to WASM mods.
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Get available capacity
    pub fn available(&self) -> usize {
        RESPONSE_BUFFER_SIZE - self.len
    }

    /// Get the total capacity
    pub fn capacity(&self) -> usize {
        RESPONSE_BUFFER_SIZE
    }
}

/// Error when the response buffer is full
#[derive(Debug, Clone)]
pub struct BufferFullError {
    /// Bytes needed
    pub needed: usize,
    /// Bytes available
    pub available: usize,
}

impl std::fmt::Display for BufferFullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Response buffer full: needed {} bytes, only {} available",
            self.needed, self.available
        )
    }
}

impl std::error::Error for BufferFullError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_buffer_new() {
        let buffer = ResponseBuffer::new();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.capacity(), RESPONSE_BUFFER_SIZE);
    }

    #[test]
    fn test_response_buffer_write() {
        let mut buffer = ResponseBuffer::new();
        let data = b"Hello, World!";

        let written = buffer.write(data);
        assert_eq!(written, data.len());
        assert_eq!(buffer.len(), data.len());
        assert_eq!(buffer.as_slice(), data);
    }

    #[test]
    fn test_response_buffer_write_multiple() {
        let mut buffer = ResponseBuffer::new();

        buffer.write(b"Hello, ");
        buffer.write(b"World!");

        assert_eq!(buffer.as_slice(), b"Hello, World!");
    }

    #[test]
    fn test_response_buffer_clear() {
        let mut buffer = ResponseBuffer::new();
        buffer.write(b"test data");

        assert!(!buffer.is_empty());
        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_response_buffer_write_all() {
        let mut buffer = ResponseBuffer::new();
        let data = b"test data";

        assert!(buffer.write_all(data).is_ok());
        assert_eq!(buffer.as_slice(), data);
    }

    #[test]
    fn test_response_buffer_write_all_overflow() {
        let mut buffer = ResponseBuffer::new();
        let data = vec![0u8; RESPONSE_BUFFER_SIZE + 1];

        let result = buffer.write_all(&data);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.needed, RESPONSE_BUFFER_SIZE + 1);
        assert_eq!(err.available, RESPONSE_BUFFER_SIZE);
    }

    #[test]
    fn test_response_buffer_partial_write() {
        let mut buffer = ResponseBuffer::new();
        // Fill most of the buffer
        buffer.write(&vec![0u8; RESPONSE_BUFFER_SIZE - 10]);

        // Try to write more than available
        let data = vec![1u8; 20];
        let written = buffer.write(&data);

        assert_eq!(written, 10); // Only 10 bytes fit
        assert_eq!(buffer.len(), RESPONSE_BUFFER_SIZE);
    }

    #[test]
    fn test_response_buffer_available() {
        let mut buffer = ResponseBuffer::new();
        assert_eq!(buffer.available(), RESPONSE_BUFFER_SIZE);

        buffer.write(&[0u8; 100]);
        assert_eq!(buffer.available(), RESPONSE_BUFFER_SIZE - 100);
    }
}
