//! Manifest V3 Chrome extension runtime, packaging, and lifecycle management.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors occurring during manifest parsing and validation.
#[derive(Debug, PartialEq, Eq)]
pub enum ManifestError {
    JsonError(String),
    UnsupportedVersion(u32),
    MissingField(&'static str),
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonError(err) => write!(f, "manifest JSON syntax error: {err}"),
            Self::UnsupportedVersion(ver) => {
                write!(
                    f,
                    "unsupported manifest version {ver}: Moonlight requires Manifest V3"
                )
            }
            Self::MissingField(field) => write!(f, "missing required manifest field: '{field}'"),
        }
    }
}

impl std::error::Error for ManifestError {}

/// Action definition in Manifest V3.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ExtensionAction {
    pub default_popup: Option<String>,
    pub default_title: Option<String>,
    pub default_icon: Option<serde_json::Value>,
}

/// Background service worker configuration in Manifest V3.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ExtensionBackground {
    pub service_worker: String,
    #[serde(rename = "type")]
    pub worker_type: Option<String>,
}

/// Content script entry in Manifest V3.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ContentScript {
    pub matches: Vec<String>,
    #[serde(default)]
    pub js: Vec<String>,
    #[serde(default)]
    pub css: Vec<String>,
    pub run_at: Option<String>,
}

/// Parsed Manifest V3 specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestV3 {
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub host_permissions: Vec<String>,
    #[serde(default)]
    pub optional_permissions: Vec<String>,
    pub action: Option<ExtensionAction>,
    pub background: Option<ExtensionBackground>,
    #[serde(default)]
    pub content_scripts: Vec<ContentScript>,
    pub devtools_page: Option<String>,
    pub chrome_url_overrides: Option<serde_json::Value>,
}

/// Safely parse and validate a Manifest V3 JSON string.
pub fn parse_manifest(raw_json: &str) -> Result<ManifestV3, ManifestError> {
    let value: serde_json::Value =
        serde_json::from_str(raw_json).map_err(|e| ManifestError::JsonError(e.to_string()))?;

    let manifest_version = value
        .get("manifest_version")
        .and_then(|v| v.as_u64())
        .ok_or(ManifestError::MissingField("manifest_version"))? as u32;

    if manifest_version != 3 {
        return Err(ManifestError::UnsupportedVersion(manifest_version));
    }

    if value.get("name").and_then(|v| v.as_str()).is_none() {
        return Err(ManifestError::MissingField("name"));
    }

    if value.get("version").and_then(|v| v.as_str()).is_none() {
        return Err(ManifestError::MissingField("version"));
    }

    serde_json::from_value(value).map_err(|e| ManifestError::JsonError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_valid_manifest_v3() {
        let json = r#"{
            "manifest_version": 3,
            "name": "Test Extension",
            "version": "1.0.0",
            "permissions": ["storage", "alarms"]
        }"#;

        let manifest = parse_manifest(json).expect("valid manifest");
        assert_eq!(manifest.manifest_version, 3);
        assert_eq!(manifest.name, "Test Extension");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.permissions, vec!["storage", "alarms"]);
    }

    #[test]
    fn test_reject_manifest_v2() {
        let json = r#"{
            "manifest_version": 2,
            "name": "Old Extension",
            "version": "1.0.0"
        }"#;

        let result = parse_manifest(json);
        assert_eq!(result, Err(ManifestError::UnsupportedVersion(2)));
    }

    #[test]
    fn test_reject_missing_required_fields() {
        let json = r#"{ "manifest_version": 3 }"#;
        assert_eq!(
            parse_manifest(json),
            Err(ManifestError::MissingField("name"))
        );

        let json_no_ver = r#"{ "manifest_version": 3, "name": "No Version" }"#;
        assert_eq!(
            parse_manifest(json_no_ver),
            Err(ManifestError::MissingField("version"))
        );
    }

    #[test]
    fn test_verify_all_20_test_extension_manifests() {
        let extensions_dir = Path::new("../../tests/extensions");
        let entries = fs::read_dir(extensions_dir)
            .or_else(|_| fs::read_dir("tests/extensions"))
            .expect("should read tests/extensions directory");

        let mut count = 0;
        for entry in entries {
            let entry = entry.expect("valid entry");
            let path = entry.path();
            if path.is_dir() {
                let manifest_file = path.join("manifest.json");
                assert!(
                    manifest_file.exists(),
                    "Missing manifest.json in {:?}",
                    path
                );
                let content = fs::read_to_string(&manifest_file)
                    .unwrap_or_else(|_| panic!("read {:?}", manifest_file));
                let manifest = parse_manifest(&content)
                    .unwrap_or_else(|e| panic!("failed to parse {:?}: {e}", manifest_file));
                assert_eq!(manifest.manifest_version, 3);
                assert!(!manifest.name.is_empty());
                assert!(!manifest.version.is_empty());
                count += 1;
            }
        }

        assert_eq!(count, 20, "Must have exactly 20 test extensions");
    }
}
