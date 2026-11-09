//! fm-tdp fuzz target: the fmn-cache entry envelope (`decode_entry_envelope`).
//!
//! Contract (§16.5): for ANY input the envelope decoder must either refuse
//! with a typed [`fmn_cache::Corrupt`]/`SerialError` refusal or return a
//! payload that is a strict slice of the input — never panic, never hang,
//! never amplify. Blob self-certification makes a forged success impossible
//! without a SHA-256 preimage, so every `Ok` here traces to bytes we wrote.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((selector, envelope)) = data.split_first() else {
        return;
    };
    let kind = match selector % 2 {
        0 => fmn_cache::EntryKind::Keyed,
        _ => fmn_cache::EntryKind::Blob,
    };
    // A deterministic expected address: keyed entries are looked up under
    // whatever address the caller names, so any fixed value is faithful.
    let address = fmn_hash::sha256(std::slice::from_ref(selector));
    if let Ok(payload) = fmn_cache::decode_entry_envelope(envelope, kind, &address, Limits::DEFAULT)
    {
        assert!(
            payload.len() <= envelope.len(),
            "decode returned {} payload bytes from a {}-byte envelope",
            payload.len(),
            envelope.len()
        );
    }
    // Err(_): a typed refusal — the contract's other allowed outcome.
});

use fmn_hash::Limits;
