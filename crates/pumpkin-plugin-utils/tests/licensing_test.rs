#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::uninlined_format_args
)]

use pumpkin_plugin_utils::{
    CheckLicenseResponse, CheckUpdateResponse, LicenseChecker, LicenseLease, LicenseStatus,
    PumpkinMetadata, get_metadata, init_with_metadata, metadata,
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
