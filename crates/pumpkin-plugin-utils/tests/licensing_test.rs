#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args
)]

use pumpkin_plugin_utils::{
    CheckLicenseResponse, CheckUpdateResponse, CrashReportPayload, CrashReporter, CrashlyticsError,
    LicenseChecker, LicenseLease, LicenseStatus, PumpkinMetadata, get_crash_context, get_metadata,
    init_with_metadata, metadata, scrub_crash_pii, sign_telemetry_payload, with_crash_context,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;

#[test]
fn license_checker_offline_and_grace_period() {
    let dir = tempdir().unwrap();
    let data_folder = dir.path();

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
        wasm_signature_hex: None,
    };

    let checker = LicenseChecker::new(data_folder);

    // 1. Evaluate with expired lease -> should enter GracePeriod
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

    let status = checker.evaluate_license(&metadata, 7);
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

    // 2. Evaluate with active lease -> should be Valid
    let active_lease = LicenseLease {
        plugin_name: "anti-grief".to_string(),
        license_key: Some("KEY-12345".to_string()),
        status: "valid".to_string(),
        last_verified_timestamp: now,
        expires_timestamp: now + 86400 * 7,
    };
    checker.write_cached_lease(&active_lease).unwrap();

    let status = checker.evaluate_license(&metadata, 7);
    match status {
        LicenseStatus::Valid(m) => assert_eq!(m.user_id, 500),
        other => panic!("Expected Valid, got {other:?}"),
    }
}

#[test]
fn global_init_and_metadata_access() {
    let dir = tempdir().unwrap();
    let data_folder = dir.path();

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
        wasm_signature_hex: Some("ab".repeat(64)),
    };

    // 1. Initialize globally with metadata
    let verified = init_with_metadata(original_meta, data_folder).expect("Init should succeed");
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

    let update_resp = serde_json::from_str::<CheckUpdateResponse>(
        r#"{"update_available":true,"latest_version":"4.0.0"}"#,
    )
    .unwrap();
    assert!(update_resp.update_available);
    assert_eq!(update_resp.latest_version.as_deref(), Some("4.0.0"));
}

#[test]
fn crashlytics_reporter_key_persistence_pii_scrubbing_and_signature_guard() {
    let dir = tempdir().unwrap();
    let data_folder = dir.path();

    let wit_meta = pumpkin_plugin_api::MarketplaceMetadata {
        marketplace_url: "http://127.0.0.1:0".to_string(),
        plugin_id: 42,
        plugin_name: "spleef-pro".to_string(),
        version: "1.4.0".to_string(),
        dev_id: 9,
        dev_name: "alex".to_string(),
        is_paid: true,
        user_id: 1001,
        license_key: Some("PPPP-1234-5678-90AB-CDEF".to_string()),
        issued_at: format!("2026-11-11T08:00:00Z#{}", "cd".repeat(64)),
    };

    // 1. Verify WIT MarketplaceMetadata conversion splits clean `issued_at` and extracts `wasm_signature_hex`
    let parsed_meta: PumpkinMetadata = wit_meta.into();
    assert_eq!(parsed_meta.issued_at, "2026-11-11T08:00:00Z");
    assert_eq!(
        parsed_meta.wasm_signature_hex.as_deref(),
        Some("cd".repeat(64).as_str())
    );

    // 2. Verify persistent Ed25519 identity key generation and telemetry payload signing
    let reporter = CrashReporter::new(data_folder);
    let key1 = reporter.load_or_derive_signing_key(&parsed_meta);
    let key2 = reporter.load_or_derive_signing_key(&parsed_meta);
    assert_eq!(key1.to_bytes(), key2.to_bytes());
    assert!(data_folder.join("crashlytics_identity.key").exists());

    let (pubkey_hex, sig_hex) = sign_telemetry_payload(&key1, "1762848000", b"{\"test\":true}");
    assert_eq!(pubkey_hex.len(), 64);
    assert_eq!(sig_hex.len(), 128);

    // 3. Verify PII scrubbing redacts licenses, API keys, IPs, and emails while preserving WIT paths
    let scrubbed = scrub_crash_pii(
        "Panic at pumpkin:plugin/events@0.1.0#on-join for user@example.com from 10.20.30.40 with PPPP-1234-5678-90AB-CDEF and pmk_live_1234567890",
        512,
    );
    assert!(scrubbed.contains("pumpkin:plugin/events@0.1.0#on-join"));
    assert!(scrubbed.contains("[REDACTED_EMAIL]"));
    assert!(scrubbed.contains("[REDACTED_IP]"));
    assert!(scrubbed.contains("[REDACTED_LICENSE]"));
    assert!(scrubbed.contains("[REDACTED_API_KEY]"));

    // 4. Verify scoped crash context helper
    assert_eq!(get_crash_context(), None);
    with_crash_context("on-player-join", || {
        assert_eq!(get_crash_context().as_deref(), Some("on-player-join"));
    });
    assert_eq!(get_crash_context(), None);

    // 5. Verify unsigned metadata (missing `wasm_signature_hex`) is rejected before any network call
    let mut unsigned_meta = parsed_meta;
    unsigned_meta.wasm_signature_hex = None;
    let dummy_payload = CrashReportPayload {
        metadata: Some(unsigned_meta.clone()),
        wasm_signature_hex: None,
        server_type: "pumpkin".to_string(),
        server_version: "0.2.2".to_string(),
        minecraft_version: Some("1.21.4".to_string()),
        os: Some("Linux".to_string()),
        arch: Some("x86_64".to_string()),
        trap_code: "panic".to_string(),
        entry_hook: Some("on-load".to_string()),
        error_message: "test error".to_string(),
        location: Some("src/lib.rs:10:5".to_string()),
        backtrace: None,
        occurrence_count: Some(1),
    };
    let res = reporter.send_payload(&unsigned_meta, &dummy_payload);
    assert!(matches!(res, Err(CrashlyticsError::MissingSignatureProof)));

    // 6. Verify official Marketplace URL maps crash reports to telemetry.pumpkinmc.org/v1/crash
    assert_eq!(
        CrashReporter::resolve_crash_endpoint("https://market.pumpkinmc.org"),
        "https://telemetry.pumpkinmc.org/v1/crash"
    );
    assert_eq!(
        CrashReporter::resolve_crash_endpoint("http://127.0.0.1:5000"),
        "http://127.0.0.1:5000/v1/crash"
    );
}
