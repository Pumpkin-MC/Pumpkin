//! Smoke / correctness tests for the optional `zlib-rs` flate2 backend.
//!
//! Run with:
//! ```text
//! cargo test -p pumpkin --features zlib-rs --test zlib_rs_backend
//! ```

#![cfg(feature = "zlib-rs")]

use std::io::{Read, Write};

use flate2::Compression;
use flate2::read::{DeflateDecoder, ZlibDecoder};
use flate2::write::{DeflateEncoder, ZlibEncoder};

fn zlib_roundtrip(data: &[u8], level: Compression) {
    let mut enc = ZlibEncoder::new(Vec::new(), level);
    enc.write_all(data).expect("zlib encode");
    let compressed = enc.finish().expect("zlib finish");
    assert!(
        !compressed.is_empty() || data.is_empty(),
        "compressed output empty for non-empty input"
    );

    let mut dec = ZlibDecoder::new(compressed.as_slice());
    let mut out = Vec::new();
    dec.read_to_end(&mut out).expect("zlib decode");
    assert_eq!(out, data, "zlib roundtrip mismatch");
}

fn deflate_roundtrip(data: &[u8], level: Compression) {
    let mut enc = DeflateEncoder::new(Vec::new(), level);
    enc.write_all(data).expect("deflate encode");
    let compressed = enc.finish().expect("deflate finish");

    let mut dec = DeflateDecoder::new(compressed.as_slice());
    let mut out = Vec::new();
    dec.read_to_end(&mut out).expect("deflate decode");
    assert_eq!(out, data, "deflate roundtrip mismatch");
}

#[test]
fn zlib_rs_feature_is_enabled() {
    // This file only compiles when `zlib-rs` is on; assert for clarity in logs.
    assert!(cfg!(feature = "zlib-rs"));
}

#[test]
fn zlib_empty_and_small() {
    zlib_roundtrip(b"", Compression::default());
    zlib_roundtrip(b"a", Compression::fast());
    zlib_roundtrip(b"hello pumpkin zlib-rs", Compression::default());
}

#[test]
fn zlib_levels_and_sizes() {
    let medium = (0u8..255).cycle().take(16 * 1024).collect::<Vec<_>>();
    let large = (0u8..255).cycle().take(256 * 1024).collect::<Vec<_>>();

    for level in [
        Compression::none(),
        Compression::fast(),
        Compression::default(),
        Compression::best(),
    ] {
        zlib_roundtrip(b"level-check", level);
        zlib_roundtrip(&medium, level);
        zlib_roundtrip(&large, level);
    }
}

#[test]
fn deflate_raw_roundtrip() {
    // Bedrock packet path uses raw DEFLATE (not zlib wrapper).
    let payload = b"bedrock-style-deflate-payload-0123456789".repeat(64);
    deflate_roundtrip(&payload, Compression::default());
    deflate_roundtrip(&payload, Compression::fast());
}

#[test]
fn zlib_incompressible_and_repetitive() {
    let randomish: Vec<u8> = (0u32..4096)
        .map(|i| (i.wrapping_mul(1_103_515_245) >> 16) as u8)
        .collect();
    let zeros = vec![0u8; 64 * 1024];
    let ones = vec![0xFFu8; 64 * 1024];

    zlib_roundtrip(&randomish, Compression::default());
    zlib_roundtrip(&zeros, Compression::best());
    zlib_roundtrip(&ones, Compression::fast());
}
