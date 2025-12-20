//! Build script for plix-tools
//!
//! Generates shadow-rs constants for build information embedding.

fn main() {
    // Generate shadow-rs constants
    shadow_rs::new().expect("Failed to generate shadow-rs build info");
}
