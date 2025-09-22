/// Memory protection helpers (stubs). Use zeroize crate in production.
pub fn clear_sensitive(_buf: &mut [u8]) { /* no-op */
}
