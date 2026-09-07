// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Complete BusMirror schemas backed by a JSON entry and companion DBC/LDF files.

use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use manager::{ParsedSchema, register_schema};
use serde::Deserialize;
use serde_json::{Map as JsonMap, Value as JsonValue};

use crate::decoder::payload::{BUSMIRROR_NETWORK_CAN, BUSMIRROR_NETWORK_LIN, FrameIdentity};
use crate::schema::dbc::{CompiledDbcSchema, DbcJson, load_dbc_bus};
use crate::schema::ldf::load_ldf_bus;

const DEFAULT_SIGNAL_NAME_PATTERN: &str = "{bus_name}_{msg_name}_{sig_name}";

/// Register the complete BusMirror schema parser.
pub fn register_busmirror_schema() {
    register_schema("busmirror", Arc::new(parse_busmirror_schema));
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BusMirrorDocument {
    version: String,
    #[serde(default)]
    signal_name_pattern: Option<String>,
    buses: Vec<BusMirrorBus>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BusMirrorBus {
    network_type: String,
    network_id: u8,
    name: String,
    source_file: String,
}

/// Immutable runtime state for one complete BusMirror schema.
#[derive(Clone)]
pub struct CompiledBusMirrorSchema {
    dbc: Arc<CompiledDbcSchema>,
}

impl CompiledBusMirrorSchema {
    pub(crate) fn from_dbc(dbc: Arc<CompiledDbcSchema>) -> Self {
        Self { dbc }
    }

    pub fn dbc(&self) -> &Arc<CompiledDbcSchema> {
        &self.dbc
    }
}

fn parse_busmirror_schema(
    stream_name: &str,
    props: &JsonMap<String, JsonValue>,
) -> Result<ParsedSchema, String> {
    let entry = props
        .get("schema_path")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| "busmirror schema requires `schema_path` prop".to_string())?;
    let content = std::fs::read_to_string(entry)
        .map_err(|err| format!("failed to read BusMirror schema entry `{entry}`: {err}"))?;
    let document: BusMirrorDocument = serde_json::from_str(&content)
        .map_err(|err| format!("failed to parse BusMirror schema entry `{entry}`: {err}"))?;

    if document.version != "v1" {
        return Err(format!(
            "unsupported BusMirror schema version `{}`; expected `v1`",
            document.version
        ));
    }
    if document.buses.is_empty() {
        return Err("BusMirror schema requires at least one bus".to_string());
    }

    let member_root = Path::new(entry).with_extension("");
    let mut bus_identities = HashSet::new();
    let mut frame_identities = HashSet::new();
    let mut buses = Vec::with_capacity(document.buses.len());

    for bus in document.buses {
        let network_type = network_type_id(&bus.network_type)?;
        if bus.name.is_empty() {
            return Err(format!(
                "BusMirror bus ({network_type}, {}) requires a non-empty name",
                bus.network_id
            ));
        }
        if !bus_identities.insert((network_type, bus.network_id)) {
            return Err(format!(
                "duplicate BusMirror bus identity ({network_type}, {})",
                bus.network_id
            ));
        }

        let source_path = member_path(&member_root, &bus.source_file)?;
        let source_kind = source_kind(&bus.source_file)?;
        match (network_type, source_kind) {
            (BUSMIRROR_NETWORK_CAN, SourceKind::Ldf) => {
                return Err(format!(
                    "BusMirror bus `{}` has network_type `can` but source_file `{}` is an LDF",
                    bus.name, bus.source_file
                ));
            }
            (BUSMIRROR_NETWORK_CAN, SourceKind::Dbc)
            | (BUSMIRROR_NETWORK_LIN, SourceKind::Dbc)
            | (BUSMIRROR_NETWORK_LIN, SourceKind::Ldf) => {}
            _ => {
                return Err(format!(
                    "BusMirror bus `{}` has unsupported network_type/source_file pair",
                    bus.name
                ));
            }
        }

        let encoded_bus_id = (u32::from(network_type) << 8) | u32::from(bus.network_id);
        let compiled_bus = match source_kind {
            SourceKind::Dbc => load_dbc_bus(&source_path, encoded_bus_id, bus.name.clone()),
            SourceKind::Ldf => load_ldf_bus(&source_path, encoded_bus_id, bus.name.clone()),
        }
        .map_err(|err| {
            format!(
                "failed to compile BusMirror bus `{}` from `{}`: {err}",
                bus.name, bus.source_file
            )
        })?;

        for message in &compiled_bus.messages {
            let identity = FrameIdentity::busmirror(network_type, bus.network_id, message.id);
            if !frame_identities.insert(identity) {
                return Err(format!(
                    "duplicate BusMirror frame identity for bus `{}`, message `{}` (0x{:X})",
                    bus.name, message.name, message.id
                ));
            }
        }
        buses.push(compiled_bus);
    }

    let pattern = document
        .signal_name_pattern
        .as_deref()
        .unwrap_or(DEFAULT_SIGNAL_NAME_PATTERN);
    let compiled = Arc::new(CompiledDbcSchema::new_busmirror(
        DbcJson { buses },
        pattern,
    )?);
    let schema = compiled.schema(stream_name);
    Ok((
        schema,
        None,
        Some(Arc::new(CompiledBusMirrorSchema::from_dbc(compiled))),
    ))
}

fn network_type_id(value: &str) -> Result<u8, String> {
    match value {
        "can" => Ok(BUSMIRROR_NETWORK_CAN),
        "lin" => Ok(BUSMIRROR_NETWORK_LIN),
        other => Err(format!(
            "unsupported BusMirror network type `{other}`; expected `can` or `lin`"
        )),
    }
}

#[derive(Clone, Copy)]
enum SourceKind {
    Dbc,
    Ldf,
}

fn source_kind(relative: &str) -> Result<SourceKind, String> {
    match Path::new(relative)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some(extension) if extension.eq_ignore_ascii_case("dbc") => Ok(SourceKind::Dbc),
        Some(extension) if extension.eq_ignore_ascii_case("ldf") => Ok(SourceKind::Ldf),
        _ => Err(format!(
            "BusMirror member `{relative}` must have a `.dbc` or `.ldf` extension"
        )),
    }
}

fn member_path(member_root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if path.as_os_str().is_empty()
        || relative.contains('\\')
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "BusMirror source_file `{relative}` must be a relative path"
        ));
    }
    source_kind(relative)?;
    let resolved = member_root.join(path);
    let metadata = std::fs::metadata(&resolved).map_err(|err| {
        format!(
            "failed to access BusMirror member `{relative}` at `{}`: {err}",
            resolved.display()
        )
    })?;
    if !metadata.is_file() {
        return Err(format!(
            "BusMirror member `{relative}` is not a regular file"
        ));
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    const BODY_LDF: &str = r#"
LIN_description_file;
LIN_protocol_version = "2.2";
LIN_language_version = "2.2";
LIN_speed = 19.2 kbps;
Signals {
  Temp: 8, 0, Body, Gateway;
}
Frames {
  LinNode1: 0x12, Body, 1 {
    Temp, 0;
  }
}
"#;

    fn temp_schema_source() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "veloflux-busmirror-schema-{}-{unique}",
            std::process::id()
        ))
    }

    fn write_entry(root: &Path, json: &str, members: &[(&str, &[u8])]) -> PathBuf {
        let entry = root.join("vehicle.json");
        let companion = root.join("vehicle");
        std::fs::create_dir_all(&companion).expect("create companion");
        std::fs::write(&entry, json).expect("write entry");
        for (name, bytes) in members {
            std::fs::write(companion.join(name), bytes).expect("write member");
        }
        entry
    }

    fn parse_entry(entry: &Path) -> Result<ParsedSchema, String> {
        let props = JsonMap::from_iter([(
            "schema_path".to_string(),
            JsonValue::String(entry.to_string_lossy().into_owned()),
        )]);
        parse_busmirror_schema("vehicle", &props)
    }

    #[test]
    fn compiles_mixed_dbc_and_ldf_members() {
        let root = temp_schema_source();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/1_TestBus.dbc");
        let dbc = std::fs::read(&fixture).expect("read DBC");
        let entry = write_entry(
            &root,
            r#"{
                "version": "v1",
                "signal_name_pattern": "{network_id}__{msg_id_hex_upper}__{sig_name}",
                "buses": [
                    { "network_type": "can", "network_id": 1, "name": "Powertrain", "source_file": "powertrain.dbc" },
                    { "network_type": "lin", "network_id": 2, "name": "Body", "source_file": "body.ldf" }
                ]
            }"#,
            &[
                ("powertrain.dbc", dbc.as_slice()),
                ("body.ldf", BODY_LDF.as_bytes()),
            ],
        );

        let (schema, _, artifact) = parse_entry(&entry).expect("compile BusMirror schema");

        assert!(schema.contains_column("1__100__StandardUnsigned"));
        assert!(schema.contains_column("2__12__Temp"));
        assert!(
            artifact
                .and_then(|value| value.downcast::<CompiledBusMirrorSchema>().ok())
                .is_some()
        );
        std::fs::remove_dir_all(root).expect("remove test source");
    }

    #[test]
    fn still_accepts_dbc_on_lin_buses() {
        let root = temp_schema_source();
        let dbc = br#"VERSION ""

NS_ :

BS_:

BU_:

BO_ 18 LinNode1: 1 Vector__XXX
 SG_ Temp : 0|8@1+ (1,0) [0|255] "" Vector__XXX
"#;
        let entry = write_entry(
            &root,
            r#"{
                "version": "v1",
                "signal_name_pattern": "{network_id}__{msg_id_hex_upper}__{sig_name}",
                "buses": [
                    { "network_type": "lin", "network_id": 2, "name": "Body", "source_file": "body.dbc" }
                ]
            }"#,
            &[("body.dbc", dbc.as_slice())],
        );

        let (schema, _, _) = parse_entry(&entry).expect("compile LIN DBC");
        assert!(schema.contains_column("2__12__Temp"));
        std::fs::remove_dir_all(root).expect("remove test source");
    }

    #[test]
    fn rejects_ldf_on_can_buses() {
        let root = temp_schema_source();
        let entry = write_entry(
            &root,
            r#"{
                "version": "v1",
                "buses": [
                    { "network_type": "can", "network_id": 1, "name": "Body", "source_file": "body.ldf" }
                ]
            }"#,
            &[("body.ldf", BODY_LDF.as_bytes())],
        );

        let err = parse_entry(&entry).expect_err("CAN LDF");
        assert!(
            err.contains("network_type `can`") && err.contains("LDF"),
            "{err}"
        );
        std::fs::remove_dir_all(root).expect("remove test source");
    }

    #[test]
    fn rejects_legacy_dbc_field_and_format_block() {
        let root = temp_schema_source();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/1_TestBus.dbc");
        let dbc = std::fs::read(&fixture).expect("read DBC");
        let entry = write_entry(
            &root,
            r#"{
                "version": "v1",
                "format": { "type": "can", "props": {} },
                "buses": [
                    { "network_type": "can", "network_id": 1, "name": "Powertrain", "dbc": "powertrain.dbc" }
                ]
            }"#,
            &[("powertrain.dbc", dbc.as_slice())],
        );

        let err = parse_entry(&entry).expect_err("legacy fields");
        assert!(
            err.contains("unknown field") || err.contains("missing field"),
            "{err}"
        );
        std::fs::remove_dir_all(root).expect("remove test source");
    }

    #[test]
    fn rejects_duplicate_normalized_lin_ids() {
        let root = temp_schema_source();
        let ldf = r#"
LIN_protocol_version = "2.2";
LIN_language_version = "2.2";
Signals { First: 8, 0, N; Second: 8, 0, N; }
Frames {
  ByID: 0x12, N, 1 { First, 0; }
  ByPID: 0x12, N, 1 { Second, 0; }
}
"#;
        let entry = write_entry(
            &root,
            r#"{
                "version": "v1",
                "buses": [
                    { "network_type": "lin", "network_id": 2, "name": "Body", "source_file": "body.ldf" }
                ]
            }"#,
            &[("body.ldf", ldf.as_bytes())],
        );

        let err = parse_entry(&entry).expect_err("duplicate LIN id");
        assert!(err.contains("duplicate"), "{err}");
        std::fs::remove_dir_all(root).expect("remove test source");
    }

    #[test]
    fn rejects_dbc_lin_ids_that_collapse_after_pid_mask() {
        let root = temp_schema_source();
        let dbc = br#"VERSION ""

NS_ :

BS_:

BU_:

BO_ 18 ByID: 1 Vector__XXX
 SG_ First : 0|8@1+ (1,0) [0|255] "" Vector__XXX
BO_ 146 ByPID: 1 Vector__XXX
 SG_ Second : 0|8@1+ (1,0) [0|255] "" Vector__XXX
"#;
        let entry = write_entry(
            &root,
            r#"{
                "version": "v1",
                "buses": [
                    { "network_type": "lin", "network_id": 2, "name": "Body", "source_file": "body.dbc" }
                ]
            }"#,
            &[("body.dbc", dbc.as_slice())],
        );

        let err = parse_entry(&entry).expect_err("normalized LIN collision");
        assert!(err.contains("duplicate BusMirror frame identity"), "{err}");
        std::fs::remove_dir_all(root).expect("remove test source");
    }
}
