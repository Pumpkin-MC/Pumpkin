#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args
)]

use ed25519_dalek::Signer;
use pumpkin_plugin_utils::{
    CheckLicenseResponse, LicenseChecker, LicenseLease, LicenseStatus, PumpkinMetadata,
    WasmSignatureEnvelope, extract_sections, get_metadata, init_with_bytes, metadata,
    strip_pumpkin_sections, verify_signature,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;
use wasm_encoder::{CustomSection, Encode, Module};

/// Helper to build a signed WASM module with `pumpkin.metadata` and `wasm_signature`.
fn create_signed_wasm(
    metadata: &PumpkinMetadata,
    signing_key: &ed25519_dalek::SigningKey,
) -> (Vec<u8>, String) {
    let module = Module::new();
    let clean_wasm = module.finish();

    let meta_json = serde_json::to_vec(metadata).unwrap();

    let mut sign_payload = Vec::new();
    sign_payload.extend_from_slice(&clean_wasm);
    sign_payload.extend_from_slice(&meta_json);

    let signature = signing_key.sign(&sign_payload);
    let pub_key_hex = hex::encode(signing_key.verifying_key().to_bytes());

    let envelope = WasmSignatureEnvelope {
        version: 1,
        algorithm: "Ed25519".to_string(),
        public_key_hex: pub_key_hex.clone(),
        signature_hex: hex::encode(signature.to_bytes()),
    };
    let sig_json = serde_json::to_vec(&envelope).unwrap();

    let mut out = clean_wasm;
    let meta_section = CustomSection {
        name: "pumpkin.metadata".into(),
        data: meta_json.as_slice().into(),
    };
    let sig_section = CustomSection {
        name: "wasm_signature".into(),
        data: sig_json.as_slice().into(),
    };

    meta_section.encode(&mut out);
    sig_section.encode(&mut out);

    (out, pub_key_hex)
}

#[test]
fn wasm_custom_section_extraction_and_stripping() {
    let seed = [7u8; 32];
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);

    let metadata = PumpkinMetadata {
        marketplace_url: "http://127.0.0.1:0".to_string(),
        plugin_id: 42,
        plugin_name: "super-shop".to_string(),
        version: "1.0.0".to_string(),
        dev_id: 10,
        dev_name: "alex".to_string(),
        is_paid: true,
        user_id: 101,
        license_key: Some("LIC-9999".to_string()),
        issued_at: "2026-08-17T08:00:00Z".to_string(),
    };

    let (wasm_bytes, pub_key_hex) = create_signed_wasm(&metadata, &signing_key);

    let extracted = extract_sections(&wasm_bytes).expect("Should extract custom sections");
    assert_eq!(extracted.metadata, metadata);
    assert_eq!(extracted.signature_envelope.public_key_hex, pub_key_hex);

    let stripped = strip_pumpkin_sections(&wasm_bytes).expect("Should strip sections");
    assert_eq!(stripped, extracted.clean_wasm);

    let res = verify_signature(
        &extracted.clean_wasm,
        &extracted.metadata_raw,
        &extracted.signature_envelope,
        &pub_key_hex,
    );
    assert!(res.is_ok());
}

#[test]
fn license_checker_offline_and_grace_period() {
    let dir = tempdir().unwrap();
    let data_folder = dir.path();

    let seed = [9u8; 32];
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);

    let metadata = PumpkinMetadata {
        marketplace_url: "http://127.0.0.1:0".to_string(),
        plugin_id: 123,
        plugin_name: "anti-grief".to_string(),
        version: "1.0.0".to_string(),
        dev_id: 5,
        dev_name: "dev".to_string(),
        is_paid: true,
        user_id: 500,
        license_key: Some("KEY-12345".to_string()),
        issued_at: "2026-08-17T08:00:00Z".to_string(),
    };

    let (wasm_bytes, _pub_key_hex) = create_signed_wasm(&metadata, &signing_key);

    let checker = LicenseChecker::new(data_folder);

    // 1. Verify offline directly
    let verified_meta = checker.verify_offline(&wasm_bytes).unwrap();
    assert_eq!(verified_meta.user_id, 500);

    // 2. Evaluate with expired lease -> should enter GracePeriod
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let lease = LicenseLease {
        plugin_name: "anti-grief".to_string(),
        license_key: Some("KEY-12345".to_string()),
        status: "valid".to_string(),
        last_verified_timestamp: now - 86400 * 2, // 2 days ago
        expires_timestamp: now - 3600,            // Expired 1 hour ago
    };
    checker.write_cached_lease(&lease).unwrap();

    let status = checker.evaluate_license(&wasm_bytes, 7);
    match status {
        LicenseStatus::GracePeriod {
            metadata: m,
            days_remaining,
            ..
        } => {
            assert_eq!(m.plugin_id, 123);
            assert!(days_remaining <= 5);
        }
        other => panic!("Expected GracePeriod, got {other:?}"),
    }

    // 3. Evaluate with active lease -> should be Valid
    let active_lease = LicenseLease {
        plugin_name: "anti-grief".to_string(),
        license_key: Some("KEY-12345".to_string()),
        status: "valid".to_string(),
        last_verified_timestamp: now,
        expires_timestamp: now + 86400 * 7,
    };
    checker.write_cached_lease(&active_lease).unwrap();

    let status = checker.evaluate_license(&wasm_bytes, 7);
    match status {
        LicenseStatus::Valid(m) => assert_eq!(m.user_id, 500),
        other => panic!("Expected Valid, got {other:?}"),
    }
}

#[test]
fn global_init_and_metadata_access() {
    let dir = tempdir().unwrap();
    let data_folder = dir.path();

    let seed = [11u8; 32];
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);

    let original_meta = PumpkinMetadata {
        marketplace_url: "http://127.0.0.1:0".to_string(),
        plugin_id: 777,
        plugin_name: "economy-core".to_string(),
        version: "3.2.1".to_string(),
        dev_id: 20,
        dev_name: "pumpkin-dev".to_string(),
        is_paid: true,
        user_id: 8888,
        license_key: Some("KEY-8888".to_string()),
        issued_at: "2026-08-17T08:00:00Z".to_string(),
    };

    let (wasm_bytes, _pub_key_hex) = create_signed_wasm(&original_meta, &signing_key);

    // 1. Initialize globally with bytes
    let verified = init_with_bytes(&wasm_bytes, data_folder).expect("Init should succeed");
    assert_eq!(verified.plugin_name, "economy-core");
    assert_eq!(verified.version, "3.2.1");
    assert_eq!(verified.user_id, 8888);

    // 2. Global metadata access
    assert_eq!(get_metadata().unwrap().plugin_id, 777);
    assert_eq!(metadata().unwrap().dev_name, "pumpkin-dev");

    // 3. Verify check license models deserialize properly
    let check_resp =
        serde_json::from_str::<CheckLicenseResponse>(r#"{"valid":true,"status":"valid"}"#).unwrap();
    assert!(check_resp.valid);
    assert_eq!(check_resp.status, "valid");
}
