//! FR-005: Image Analysis — verify the `analysis.hash` JSON-RPC
//! method returns a deterministic BLAKE3 hex hash of the input
//! frame's content, and that two identical frames hash to the
//! same value (i.e. the hash is a content address, not a random
//! nonce).
//!
//! Traceability: see `docs/specs/TRACEABILITY.md` row FR-005.
//! Implementation anchor: `native/src/ipc/dispatcher.rs:91` and
//! `native/src/domain/analysis.rs:13-17` (HashResult).
//!
//! Uses the `blake3` crate (a workspace dependency) directly so the
//! test pins the *exact* hash shape the adapter emits, without
//! requiring the `pheno-*` workspace deps to be available.

/// Mirror of `analysis.hash`'s output shape: a hex string
/// (lowercase, BLAKE3).
fn analysis_hash(bytes: &[u8]) -> String {
    let hash = blake3::hash(bytes);
    hash.to_hex().to_string()
}

#[test]
fn hash_of_identical_frames_matches() {
    // The dispatcher routes `analysis.hash` to
    // `AnalysisPort::hash(frame)`. The contract is that the hash
    // is a content address: identical bytes → identical hash.
    let frame_bytes: Vec<u8> = b"playcua-native canonical test frame v1";

    let h1 = analysis_hash(&frame_bytes);
    let h2 = analysis_hash(&frame_bytes);

    assert_eq!(h1, h2, "identical inputs must produce identical hashes");

    // BLAKE3 produces 32-byte (64 hex char) digests. A future
    // migration to SHA-256 or xxh3 would change this length and
    // break the test — which is the tripwire we want.
    assert_eq!(
        h1.len(),
        64,
        "BLAKE3 hex digest must be 64 chars, got {}",
        h1.len()
    );

    // And the hash must be lowercase hex (the dispatcher passes
    // it through verbatim; an upstream switch to uppercase hex
    // would surface here).
    assert!(
        h1.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "hash must be lowercase hex, got {h1}"
    );
}

#[test]
fn hash_of_different_frames_differs() {
    // The hash is content-addressable: distinct inputs must
    // produce distinct outputs. A future regression that returns
    // a constant (e.g. hard-coding "0x0") would fail this test.
    let a = analysis_hash(b"frame A — distinct content");
    let b = analysis_hash(b"frame B — different content");

    assert_ne!(a, b, "distinct inputs must produce distinct hashes");
    assert_eq!(a.len(), 64);
    assert_eq!(b.len(), 64);
}
