[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
![Apache 2.0][license-image]

# Uint-ZigZag
Uint-zigzag is a convenience wrapper for zig-zag encoding integers to byte sequences.

This allows better compression since the majority of numbers are quite small resulting
in 1 or 2 bytes in the most common case vs 4 for 32-bit numbers or 8 for 64-bit numbers.

This also permits the user to not have to think about which integer type is the most efficient to compress.

This crate is passively maintained.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/uint-zigzag.svg
[crate-link]: https://crates.io/crates/uint-zigzag
[docs-image]: https://docs.rs/uint-zigzag/badge.svg
[docs-link]: https://docs.rs/uint-zigzag/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg