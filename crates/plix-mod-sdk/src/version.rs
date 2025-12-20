//! SDK version constants and compatibility checking
//!
//! The SDK uses two version numbers:
//! - **ABI version**: Binary interface compatibility (must match runtime exactly)
//! - **API version**: Source-level API compatibility (forward-compatible within major version)

use crate::abi;
use crate::error::{ErrorCode, ModError};

/// Current SDK ABI version (must match runtime ABI version)
pub const SDK_ABI_VERSION: u8 = 1;

/// Current SDK API version (source-level compatibility)
pub const SDK_API_VERSION: u8 = 1;

/// Check if the SDK is compatible with the runtime
///
/// Returns `true` if the runtime ABI version matches the SDK ABI version.
/// A mismatch means the mod was compiled against a different runtime version.
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::version::check_compatibility;
///
/// if !check_compatibility() {
///     warn!("SDK/runtime version mismatch - some features may not work");
/// }
/// ```
pub fn check_compatibility() -> bool {
    let runtime_abi = abi::get_abi_version();
    let runtime_api = abi::get_api_version();

    // ABI must match exactly
    if runtime_abi != SDK_ABI_VERSION as u32 {
        return false;
    }

    // API version: runtime must be >= SDK version (forward compatible)
    if runtime_api < SDK_API_VERSION as u32 {
        return false;
    }

    true
}

/// Assert SDK/runtime compatibility, logging a warning if mismatched
///
/// This function should be called during mod initialization. If there's
/// a version mismatch, it logs a warning but allows execution to continue
/// (graceful degradation).
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::version::assert_compatible;
///
/// fn init() {
///     assert_compatible();
///     // ... rest of initialization
/// }
/// ```
pub fn assert_compatible() {
    if !check_compatibility() {
        let runtime_abi = abi::get_abi_version();
        let runtime_api = abi::get_api_version();
        crate::log::warn(&alloc::format!(
            "SDK/runtime version mismatch: SDK ABI={} API={}, Runtime ABI={} API={}",
            SDK_ABI_VERSION,
            SDK_API_VERSION,
            runtime_abi,
            runtime_api
        ));
    }
}

/// Get the runtime's ABI version
pub fn get_runtime_abi_version() -> u32 {
    abi::get_abi_version()
}

/// Get the runtime's API version
pub fn get_runtime_api_version() -> u32 {
    abi::get_api_version()
}

/// Check version and return an error if incompatible
pub fn require_compatible() -> crate::error::Result<()> {
    if !check_compatibility() {
        let runtime_abi = abi::get_abi_version();
        let runtime_api = abi::get_api_version();
        return Err(ModError::new(
            ErrorCode::Unsupported,
            alloc::format!(
                "SDK/runtime version mismatch: SDK ABI={} API={}, Runtime ABI={} API={}",
                SDK_ABI_VERSION,
                SDK_API_VERSION,
                runtime_abi,
                runtime_api
            ),
        ));
    }
    Ok(())
}
