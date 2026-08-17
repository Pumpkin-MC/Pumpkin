//! WASM binary inspection and custom section extraction.

use crate::models::{
    LEGACY_SIGNATURE_SECTION, PUMPKIN_METADATA_SECTION, PumpkinMetadata, WASM_SIGNATURE_SECTION,
    WasmSignatureEnvelope,
};
use thiserror::Error;
use wasmparser::{Parser, Payload};

/// Errors encountered while parsing or inspecting WASM binaries.
#[derive(Debug, Error)]
pub enum WasmError {
    /// Error reading the WASM file from disk.
    #[error("Failed to read plugin WASM file from disk: {0}")]
    Io(#[from] std::io::Error),
    /// Error parsing the WASM structure.
    #[error("Failed to parse WASM binary: {0}")]
    Parser(String),
    /// Metadata custom section is missing.
    #[error("Missing 'pumpkin.metadata' custom section in WASM binary")]
    MissingMetadata,
    /// Signature custom section is missing.
    #[error("Missing 'wasm_signature' custom section in WASM binary")]
    MissingSignature,
    /// Failed to deserialize metadata JSON.
    #[error("Failed to deserialize Pumpkin metadata JSON: {0}")]
    InvalidMetadataJson(#[from] serde_json::Error),
}

/// Decodes an unsigned LEB128 integer from a byte slice.
fn read_leb128(bytes: &[u8]) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        if shift >= 64 {
            return None;
        }
        result |= u64::from(byte & 0x7f) << shift;
        if (byte & 0x80) == 0 {
            return Some((result, i + 1));
        }
        shift += 7;
    }
    None
}

/// Strips `pumpkin.metadata`, `wasm_signature`, and legacy signature sections from WASM bytes.
///
/// Returns the "clean" WASM payload used for signature calculation and verification.
///
/// # Errors
///
/// Returns a `WasmError` if the WASM binary is invalid.
pub fn strip_pumpkin_sections(wasm_bytes: &[u8]) -> Result<Vec<u8>, WasmError> {
    let mut last_valid_end = wasm_bytes.len();
    let parser = Parser::new(0);

    for payload in parser.parse_all(wasm_bytes) {
        match payload {
            Ok(Payload::Version { ref range, .. }) => {
                last_valid_end = range.end;
            }
            Ok(Payload::CustomSection(cs)) => {
                if cs.name() == PUMPKIN_METADATA_SECTION
                    || cs.name() == WASM_SIGNATURE_SECTION
                    || cs.name() == LEGACY_SIGNATURE_SECTION
                {
                    // Section will be omitted
                } else {
                    last_valid_end = cs.range().end;
                }
            }
            Ok(p) => {
                if let Some((_, range)) = p.as_section() {
                    last_valid_end = range.end;
                }
            }
            Err(_) => {
                // Stop parsing on trailing appended sections
                break;
            }
        }
    }

    Ok(wasm_bytes[..last_valid_end].to_vec())
}

/// Extracted custom sections and clean WASM payload.
#[derive(Debug, Clone)]
pub struct ExtractedSections {
    /// Parsed pumpkin metadata.
    pub metadata: PumpkinMetadata,
    /// Raw metadata JSON bytes (used for signature verification).
    pub metadata_raw: Vec<u8>,
    /// Parsed signature envelope.
    pub signature_envelope: WasmSignatureEnvelope,
    /// Clean WASM bytes without metadata and signature custom sections.
    pub clean_wasm: Vec<u8>,
}

/// Extracts metadata and signature custom sections from a WASM binary.
///
/// # Errors
///
/// Returns a `WasmError` if sections are missing or malformed.
pub fn extract_sections(wasm_bytes: &[u8]) -> Result<ExtractedSections, WasmError> {
    let clean_wasm = strip_pumpkin_sections(wasm_bytes)?;

    let mut metadata_raw: Option<Vec<u8>> = None;
    let mut signature_raw: Option<Vec<u8>> = None;

    let parser = Parser::new(0);
    for payload in parser.parse_all(wasm_bytes) {
        if let Ok(Payload::CustomSection(cs)) = payload {
            if cs.name() == PUMPKIN_METADATA_SECTION {
                metadata_raw = Some(cs.data().to_vec());
            } else if cs.name() == WASM_SIGNATURE_SECTION || cs.name() == LEGACY_SIGNATURE_SECTION {
                signature_raw = Some(cs.data().to_vec());
            }
        }
    }

    // Check trailing sections if not found in primary pass (e.g. appended custom sections)
    if metadata_raw.is_none() || signature_raw.is_none() {
        if clean_wasm.len() < wasm_bytes.len() {
            let trailing = &wasm_bytes[clean_wasm.len()..];
            let mut cursor = 0;
            while cursor < trailing.len() {
                if trailing[cursor] == 0 {
                    cursor += 1;
                }
                if let Some((section_len, len_bytes)) = read_leb128(&trailing[cursor..]) {
                    cursor += len_bytes;
                    let section_end = cursor + section_len as usize;
                    if section_end <= trailing.len() {
                        let section_data = &trailing[cursor..section_end];
                        if let Some((name_len, name_len_bytes)) = read_leb128(section_data) {
                            let name_start = name_len_bytes;
                            let name_end = name_start + name_len as usize;
                            if name_end <= section_data.len()
                                && let Ok(name) =
                                    std::str::from_utf8(&section_data[name_start..name_end])
                            {
                                let payload_data = &section_data[name_end..];
                                if name == PUMPKIN_METADATA_SECTION {
                                    metadata_raw = Some(payload_data.to_vec());
                                } else if name == WASM_SIGNATURE_SECTION
                                    || name == LEGACY_SIGNATURE_SECTION
                                {
                                    signature_raw = Some(payload_data.to_vec());
                                }
                            }
                        }
                    }
                    cursor = section_end;
                    continue;
                }
                break;
            }
        }
    }

    let meta_bytes = metadata_raw.ok_or(WasmError::MissingMetadata)?;
    let sig_bytes = signature_raw.ok_or(WasmError::MissingSignature)?;

    let metadata: PumpkinMetadata = serde_json::from_slice(&meta_bytes)?;

    let signature_envelope: WasmSignatureEnvelope =
        if let Ok(envelope) = serde_json::from_slice::<WasmSignatureEnvelope>(&sig_bytes) {
            envelope
        } else {
            // Legacy raw signature fallback
            WasmSignatureEnvelope {
                version: 1,
                algorithm: "Ed25519".to_string(),
                public_key_hex: String::new(),
                signature_hex: hex::encode(&sig_bytes),
            }
        };

    Ok(ExtractedSections {
        metadata,
        metadata_raw: meta_bytes,
        signature_envelope,
        clean_wasm,
    })
}

/// Automatically locates and reads the plugin's own WASM file from the filesystem.
///
/// Discovers candidates by:
/// 1. Checking data folder path naming hint (e.g. `plugins/data/<name>` -> `plugins/<name>.wasm`).
/// 2. Scanning the `plugins/` directory for `.wasm` files containing valid pumpkin custom sections.
/// 3. Scanning current working directory for `.wasm` files.
///
/// # Errors
///
/// Returns `WasmError::Io` if no valid WASM binary can be found or read.
pub fn find_self_wasm(data_folder: &std::path::Path) -> Result<Vec<u8>, WasmError> {
    // 1. Check data folder hint
    if let Some(folder_name) = data_folder.file_name().and_then(|n| n.to_str()) {
        if folder_name != "data" && !folder_name.is_empty() {
            let candidate_paths = [
                format!("plugins/{folder_name}.wasm"),
                format!("plugins/{folder_name}.cwasm"),
                format!("plugins/{folder_name}"),
                format!("{folder_name}.wasm"),
            ];
            for path in &candidate_paths {
                if let Ok(bytes) = std::fs::read(path) {
                    return Ok(bytes);
                }
            }
        }
    }

    // 2. Scan `plugins/` directory
    if let Ok(entries) = std::fs::read_dir("plugins") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "wasm" || ext == "cwasm" {
                    if let Ok(bytes) = std::fs::read(&path) {
                        // Check if this WASM file contains pumpkin metadata
                        if extract_sections(&bytes).is_ok() {
                            return Ok(bytes);
                        }
                    }
                }
            }
        }
    }

    // 3. Scan current directory
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "wasm" || ext == "cwasm" {
                    if let Ok(bytes) = std::fs::read(&path) {
                        if extract_sections(&bytes).is_ok() {
                            return Ok(bytes);
                        }
                    }
                }
            }
        }
    }

    Err(WasmError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Could not locate plugin WASM binary with pumpkin metadata in plugins directory",
    )))
}
