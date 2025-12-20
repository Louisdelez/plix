//! WASM linear memory access helpers
//!
//! Safe wrappers for reading from and writing to WASM module memory.
//! All operations include bounds checking to prevent OOB access.

use crate::errors::RuntimeError;
use wasmtime::{AsContextMut, Memory};

/// Read bytes from WASM linear memory with bounds checking
///
/// # Arguments
/// * `store` - Mutable store context
/// * `memory` - The WASM memory to read from
/// * `offset` - Byte offset in memory
/// * `length` - Number of bytes to read
///
/// # Returns
/// A vector containing the read bytes, or an error if out of bounds
pub fn read_bytes(
    store: impl AsContextMut,
    memory: &Memory,
    offset: usize,
    length: usize,
) -> Result<Vec<u8>, RuntimeError> {
    let ctx = store.as_context();
    let mem_data = memory.data(&ctx);
    let mem_size = mem_data.len();

    // Check bounds
    let end = offset
        .checked_add(length)
        .ok_or(RuntimeError::MemoryAccessOutOfBounds {
            offset,
            length,
            memory_size: mem_size,
        })?;

    if end > mem_size {
        return Err(RuntimeError::MemoryAccessOutOfBounds {
            offset,
            length,
            memory_size: mem_size,
        });
    }

    Ok(mem_data[offset..end].to_vec())
}

/// Read a string from WASM linear memory
///
/// # Arguments
/// * `store` - Mutable store context
/// * `memory` - The WASM memory to read from
/// * `offset` - Byte offset in memory
/// * `length` - Number of bytes to read
///
/// # Returns
/// A UTF-8 string, or an error if out of bounds or invalid UTF-8
pub fn read_string(
    store: impl AsContextMut,
    memory: &Memory,
    offset: usize,
    length: usize,
) -> Result<String, RuntimeError> {
    let bytes = read_bytes(store, memory, offset, length)?;
    String::from_utf8(bytes)
        .map_err(|e| RuntimeError::HostFunction(format!("Invalid UTF-8: {}", e)))
}

/// Read a string from WASM memory, truncating invalid UTF-8 gracefully
///
/// If the bytes are not valid UTF-8, this returns a lossy conversion
/// with replacement characters for invalid sequences.
pub fn read_string_lossy(
    store: impl AsContextMut,
    memory: &Memory,
    offset: usize,
    length: usize,
) -> Result<String, RuntimeError> {
    let bytes = read_bytes(store, memory, offset, length)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Write bytes to WASM linear memory with bounds checking
///
/// # Arguments
/// * `store` - Mutable store context
/// * `memory` - The WASM memory to write to
/// * `offset` - Byte offset in memory
/// * `data` - Bytes to write
///
/// # Returns
/// Ok(()) on success, or an error if out of bounds
pub fn write_bytes(
    mut store: impl AsContextMut,
    memory: &Memory,
    offset: usize,
    data: &[u8],
) -> Result<(), RuntimeError> {
    let mem_data = memory.data_mut(&mut store);
    let mem_size = mem_data.len();
    let length = data.len();

    // Check bounds
    let end = offset
        .checked_add(length)
        .ok_or(RuntimeError::MemoryAccessOutOfBounds {
            offset,
            length,
            memory_size: mem_size,
        })?;

    if end > mem_size {
        return Err(RuntimeError::MemoryAccessOutOfBounds {
            offset,
            length,
            memory_size: mem_size,
        });
    }

    mem_data[offset..end].copy_from_slice(data);
    Ok(())
}

/// Read a little-endian i32 from WASM memory
pub fn read_i32(
    store: impl AsContextMut,
    memory: &Memory,
    offset: usize,
) -> Result<i32, RuntimeError> {
    let bytes = read_bytes(store, memory, offset, 4)?;
    Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Read a little-endian u32 from WASM memory
pub fn read_u32(
    store: impl AsContextMut,
    memory: &Memory,
    offset: usize,
) -> Result<u32, RuntimeError> {
    let bytes = read_bytes(store, memory, offset, 4)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Read a little-endian i64 from WASM memory
pub fn read_i64(
    store: impl AsContextMut,
    memory: &Memory,
    offset: usize,
) -> Result<i64, RuntimeError> {
    let bytes = read_bytes(store, memory, offset, 8)?;
    Ok(i64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

/// Read a little-endian u64 from WASM memory
pub fn read_u64(
    store: impl AsContextMut,
    memory: &Memory,
    offset: usize,
) -> Result<u64, RuntimeError> {
    let bytes = read_bytes(store, memory, offset, 8)?;
    Ok(u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

/// Write a little-endian i32 to WASM memory
pub fn write_i32(
    store: impl AsContextMut,
    memory: &Memory,
    offset: usize,
    value: i32,
) -> Result<(), RuntimeError> {
    write_bytes(store, memory, offset, &value.to_le_bytes())
}

/// Write a little-endian u32 to WASM memory
pub fn write_u32(
    store: impl AsContextMut,
    memory: &Memory,
    offset: usize,
    value: u32,
) -> Result<(), RuntimeError> {
    write_bytes(store, memory, offset, &value.to_le_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::{Engine, Store};

    fn create_test_memory() -> (Store<()>, Memory) {
        let engine = Engine::default();
        let mut store = Store::new(&engine, ());
        let memory = Memory::new(&mut store, wasmtime::MemoryType::new(1, None)).unwrap();
        (store, memory)
    }

    #[test]
    fn test_read_bytes_valid() {
        let (mut store, memory) = create_test_memory();

        // Write some test data
        memory.data_mut(&mut store)[0..4].copy_from_slice(&[1, 2, 3, 4]);

        let result = read_bytes(&mut store, &memory, 0, 4).unwrap();
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_read_bytes_out_of_bounds() {
        let (mut store, memory) = create_test_memory();
        let mem_size = memory.data_size(&store);

        // Try to read past the end
        let result = read_bytes(&mut store, &memory, mem_size - 2, 4);
        assert!(result.is_err());

        if let Err(RuntimeError::MemoryAccessOutOfBounds {
            offset,
            length,
            memory_size,
        }) = result
        {
            assert_eq!(offset, mem_size - 2);
            assert_eq!(length, 4);
            assert_eq!(memory_size, mem_size);
        } else {
            panic!("Expected MemoryAccessOutOfBounds error");
        }
    }

    #[test]
    fn test_write_bytes_valid() {
        let (mut store, memory) = create_test_memory();

        write_bytes(&mut store, &memory, 0, &[10, 20, 30, 40]).unwrap();

        let result = read_bytes(&mut store, &memory, 0, 4).unwrap();
        assert_eq!(result, vec![10, 20, 30, 40]);
    }

    #[test]
    fn test_write_bytes_out_of_bounds() {
        let (mut store, memory) = create_test_memory();
        let mem_size = memory.data_size(&store);

        let result = write_bytes(&mut store, &memory, mem_size - 2, &[1, 2, 3, 4]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_string() {
        let (mut store, memory) = create_test_memory();
        let test_str = "Hello, WASM!";

        memory.data_mut(&mut store)[0..test_str.len()].copy_from_slice(test_str.as_bytes());

        let result = read_string(&mut store, &memory, 0, test_str.len()).unwrap();
        assert_eq!(result, test_str);
    }

    #[test]
    fn test_read_string_invalid_utf8() {
        let (mut store, memory) = create_test_memory();
        // Invalid UTF-8 sequence
        memory.data_mut(&mut store)[0..4].copy_from_slice(&[0xFF, 0xFE, 0x00, 0x01]);

        let result = read_string(&mut store, &memory, 0, 4);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_string_lossy() {
        let (mut store, memory) = create_test_memory();
        // Invalid UTF-8 with some valid chars
        memory.data_mut(&mut store)[0..5].copy_from_slice(&[0x48, 0xFF, 0x69, 0xFE, 0x21]);

        let result = read_string_lossy(&mut store, &memory, 0, 5).unwrap();
        assert!(result.contains('H'));
        assert!(result.contains('i'));
        assert!(result.contains('!'));
        assert!(result.contains('�')); // Replacement character
    }

    #[test]
    fn test_read_write_i32() {
        let (mut store, memory) = create_test_memory();

        write_i32(&mut store, &memory, 0, -12345).unwrap();
        let result = read_i32(&mut store, &memory, 0).unwrap();
        assert_eq!(result, -12345);
    }

    #[test]
    fn test_read_write_u64() {
        let (mut store, memory) = create_test_memory();

        write_bytes(&mut store, &memory, 0, &0x123456789ABCDEFu64.to_le_bytes()).unwrap();
        let result = read_u64(&mut store, &memory, 0).unwrap();
        assert_eq!(result, 0x123456789ABCDEF);
    }

    #[test]
    fn test_overflow_check() {
        let (mut store, memory) = create_test_memory();

        // Try to trigger integer overflow in bounds check
        let result = read_bytes(&mut store, &memory, usize::MAX, 10);
        assert!(result.is_err());
    }
}
