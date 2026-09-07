// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! LIN Description File compiler for BusMirror and GBF schemas.
//!
//! Parses LIN 2.0–2.2A LDF files and compiles unconditional numeric frames into
//! the shared [`BusJson`](super::dbc::BusJson) model used by DBC decoding.
//! Event-triggered, sporadic, and diagnostic frames are skipped at parse time
//! and are not decoded in v1.

mod parse;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::dbc::{BusJson, MessageJson, SignalJson};
use parse::{ParsedLdf, error, parse_ldf};

/// Load one LDF file and assign the bus identity supplied by its owning schema.
pub(crate) fn load_ldf_bus(path: &Path, id: u32, name: String) -> Result<BusJson, String> {
    let data =
        fs::read(path).map_err(|err| format!("failed to read LDF at {}: {err}", path.display()))?;
    let source = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("schema.ldf");
    compile_ldf_bus(source, &data, id, name)
}

fn compile_ldf_bus(
    source_file: &str,
    data: &[u8],
    id: u32,
    name: String,
) -> Result<BusJson, String> {
    let file = parse_ldf(source_file, data)
        .map_err(|err| format!("failed to parse LDF source file: {err}"))?;
    validate_version(&file.protocol_version, "LIN_protocol_version")?;
    validate_version(&file.language_version, "LIN_language_version")?;
    Ok(BusJson {
        name: Some(name),
        id,
        messages: compile_messages(&file)?,
    })
}

fn compile_messages(file: &ParsedLdf) -> Result<Vec<MessageJson>, String> {
    let mut ids = HashMap::new();
    let mut names = HashMap::new();
    let mut messages = Vec::with_capacity(file.frames.len());
    for frame in &file.frames {
        if !frame.length_explicit && file.language_version != "2.0" {
            return Err(error(
                &file.source_file,
                frame.pos,
                format!(
                    "frame `{}` omits payload length; this is only supported by LIN 2.0",
                    frame.name
                ),
            ));
        }
        if let Some(previous) = ids.insert(frame.id, frame.name.clone()) {
            return Err(error(
                &file.source_file,
                frame.pos,
                format!(
                    "frames `{previous}` and `{}` use duplicate ID {:#x}",
                    frame.name, frame.id
                ),
            ));
        }
        if names.insert(frame.name.clone(), ()).is_some() {
            return Err(error(
                &file.source_file,
                frame.pos,
                format!("duplicate frame name `{}`", frame.name),
            ));
        }
        let mut placed = HashMap::new();
        let mut signals = Vec::with_capacity(frame.placements.len());
        for placement in &frame.placements {
            if placed.insert(placement.name.clone(), ()).is_some() {
                return Err(error(
                    &file.source_file,
                    placement.pos,
                    format!(
                        "frame `{}` places signal `{}` more than once",
                        frame.name, placement.name
                    ),
                ));
            }
            let definition = file.signals.get(&placement.name).ok_or_else(|| {
                error(
                    &file.source_file,
                    placement.pos,
                    format!(
                        "frame `{}` references undefined signal `{}`",
                        frame.name, placement.name
                    ),
                )
            })?;
            let end = u32::from(placement.start) + u32::from(definition.length);
            let frame_bits = u32::from(frame.length) * 8;
            if end > frame_bits {
                return Err(error(
                    &file.source_file,
                    placement.pos,
                    format!(
                        "frame `{}` signal `{}` ends at bit {end}, frame has {frame_bits} bits",
                        frame.name, placement.name
                    ),
                ));
            }
            if definition.length > 64 {
                return Err(error(
                    &file.source_file,
                    definition.pos,
                    format!(
                        "signal `{}` length {} exceeds numeric decoder limit 64",
                        definition.name, definition.length
                    ),
                ));
            }
            let (scale, offset) = resolve_encoding(file, &definition.name)?;
            signals.push(SignalJson {
                name: definition.name.clone(),
                start: u32::from(placement.start),
                length: u32::from(definition.length),
                scale: Some(scale),
                offset: Some(offset),
                is_big_endian: false,
                is_signed: false,
                is_multiplexer: false,
                is_multiplexed: false,
                multiplexer_value: None,
                min: None,
                max: None,
            });
        }
        signals.sort_by_key(|signal| signal.start);
        messages.push(MessageJson {
            name: frame.name.clone(),
            id: u32::from(frame.id),
            frame_id: format!("0x{:X}", frame.id),
            _length: u32::from(frame.length),
            signals,
        });
    }
    messages.sort_by_key(|message| message.id);
    Ok(messages)
}

fn validate_version(version: &str, field: &str) -> Result<(), String> {
    match version {
        "2.0" | "2.1" | "2.2" | "2.2A" => Ok(()),
        "" => Err(format!("LDF has no {field}")),
        other => Err(format!(
            "unsupported {field} `{other}`; supported versions are 2.0 through 2.2A"
        )),
    }
}

fn resolve_encoding(file: &ParsedLdf, signal_name: &str) -> Result<(f64, f64), String> {
    let Some(encoding_name) = file.representations.get(signal_name) else {
        return Ok((1.0, 0.0));
    };
    let encoding = file.encodings.get(encoding_name).ok_or_else(|| {
        format!("LDF signal `{signal_name}` references undefined encoding `{encoding_name}`")
    })?;
    if encoding.physical.is_empty() {
        if encoding.has_ascii || encoding.has_bcd {
            return Err(error(
                &file.source_file,
                encoding.pos,
                format!(
                    "signal `{signal_name}` uses unsupported ASCII/BCD encoding `{encoding_name}`"
                ),
            ));
        }
        return Ok((1.0, 0.0));
    }
    let Some(first) = encoding.physical.first() else {
        return Ok((1.0, 0.0));
    };
    let scale = first.scale;
    let offset = first.offset;
    for physical in encoding.physical.iter().skip(1) {
        if !same_float(scale, physical.scale) || !same_float(offset, physical.offset) {
            return Err(error(
                &file.source_file,
                physical.pos,
                format!(
                    "signal `{signal_name}` encoding `{encoding_name}` has non-linear physical ranges"
                ),
            ));
        }
    }
    Ok((scale, offset))
}

fn same_float(a: f64, b: f64) -> bool {
    a == b || (a - b).abs() <= 1e-12 * 1.0f64.max(a.abs()).max(b.abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOTOR_LDF: &str = r#"
LIN_description_file;
LIN_protocol_version = "2.2";
LIN_language_version = "2.2";
LIN_speed = 19.2 kbps;

Nodes {
  Master: Gateway, 5 ms, 0.1 ms;
  Slaves: Motor;
}

Node_attributes {
  Motor {
    LIN_protocol = "2.2";
    response_tolerance = 40 %;
    configurable_frames { MotorStatus; }
  }
}

Signals {
  MotorSpeed: 16, 0, Motor, Gateway;
  MotorState: 8, { 0 }, Motor, Gateway;
}

Frames {
  MotorStatus: 0x12, Motor, 8 {
    MotorSpeed, 8;
    MotorState, 24;
  }
}

Event_triggered_frames {
  StatusEvent : Collision_resolver, 0x06, MotorStatus;
}

Signal_encoding_types {
  MotorSpeedEncoding {
    logical_value, 65535, "invalid";
    physical_value, 0, 65534, 0.25, -100, "rpm";
  }
}

Signal_representation {
  MotorSpeedEncoding: MotorSpeed;
}

Schedule_tables {
  Normal {
    MotorStatus delay 10 ms;
  }
}
"#;

    fn compile(source: &str, ldf: &str) -> BusJson {
        compile_ldf_bus(source, ldf.as_bytes(), 0x200, "Body".to_string()).expect("compile LDF")
    }

    #[test]
    fn compile_unconditional_numeric_frames() {
        let bus = compile("motor.ldf", MOTOR_LDF);
        assert_eq!(bus.name.as_deref(), Some("Body"));
        assert_eq!(bus.id, 0x200);
        assert_eq!(bus.messages.len(), 1);
        let message = &bus.messages[0];
        assert_eq!(message.name, "MotorStatus");
        assert_eq!(message.id, 0x12);
        assert_eq!(message._length, 8);
        assert_eq!(message.signals.len(), 2);
        assert_eq!(message.signals[0].name, "MotorSpeed");
        assert_eq!(message.signals[0].start, 8);
        assert_eq!(message.signals[0].length, 16);
        assert_eq!(message.signals[0].scale, Some(0.25));
        assert_eq!(message.signals[0].offset, Some(-100.0));
        assert!(!message.signals[0].is_big_endian);
        assert!(!message.signals[0].is_signed);
        assert_eq!(message.signals[1].name, "MotorState");
        assert_eq!(message.signals[1].scale, Some(1.0));
        assert_eq!(message.signals[1].offset, Some(0.0));
    }

    #[test]
    fn compile_lin20_infers_frame_length_from_id() {
        let bus = compile(
            "lin20.ldf",
            r#"
LIN_protocol_version = "2.0";
LIN_language_version = "2.0";
Signals { Low: 8, 0, N; Mid: 8, 0, N; High: 8, 0, N; }
Frames {
  LowFrame: 0x01, N { Low, 0; }
  MidFrame: 0x20, N { Mid, 0; }
  HighFrame: 0x30, N { High, 0; }
}
"#,
        );
        assert_eq!(bus.messages[0]._length, 2);
        assert_eq!(bus.messages[1]._length, 4);
        assert_eq!(bus.messages[2]._length, 8);
    }

    #[test]
    fn compile_rejects_bad_definitions() {
        let cases = [
            (
                "unsupported version",
                r#"LIN_protocol_version="1.3"; LIN_language_version="1.3"; Signals{} Frames{}"#,
                r#"unsupported LIN_protocol_version `1.3`"#,
            ),
            (
                "undefined signal",
                r#"LIN_protocol_version="2.1"; LIN_language_version="2.1"; Signals{} Frames{F:1,N,1{Missing,0;}}"#,
                r#"references undefined signal `Missing`"#,
            ),
            (
                "signal overrun",
                r#"LIN_protocol_version="2.1"; LIN_language_version="2.1"; Signals{S:16,0,N;} Frames{F:1,N,1{S,0;}}"#,
                "signal `S` ends at bit 16, frame has 8 bits",
            ),
            (
                "non linear",
                r#"LIN_protocol_version="2.1"; LIN_language_version="2.1";
Signals{S:8,0,N;} Frames{F:1,N,1{S,0;}}
Signal_encoding_types{E{physical_value,0,10,1,0; physical_value,11,20,2,0;}}
Signal_representation{E:S;}"#,
                "has non-linear physical ranges",
            ),
            (
                "ascii encoding",
                r#"LIN_protocol_version="2.1"; LIN_language_version="2.1";
Signals{S:8,0,N;} Frames{F:1,N,1{S,0;}}
Signal_encoding_types{E{ascii_value;}}
Signal_representation{E:S;}"#,
                "uses unsupported ASCII/BCD encoding",
            ),
        ];
        for (name, ldf, want) in cases {
            let err =
                compile_ldf_bus("bad.ldf", ldf.as_bytes(), 0, "Bus".to_string()).expect_err(name);
            assert!(err.contains(want), "{name}: {err}");
        }
    }

    #[test]
    fn compile_upstream_version_fixtures() {
        let cases = [
            (
                "lin20.ldf",
                include_str!("ldf/testdata/upstream/lin20.ldf"),
                2,
                "VL1_CEM_Frm1",
                1,
                2,
                "InternalLightsRequest",
                0,
                2,
                1.0,
                0.0,
            ),
            (
                "lin21.ldf",
                include_str!("ldf/testdata/upstream/lin21.ldf"),
                5,
                "LSM_Frm1",
                2,
                2,
                "LeftIntLightsSwitch",
                0,
                8,
                1.0,
                100.0,
            ),
            (
                "lin22.ldf",
                include_str!("ldf/testdata/upstream/lin22.ldf"),
                5,
                "LSM_Frm1",
                2,
                2,
                "LeftIntLightsSwitch",
                8,
                8,
                1.0,
                100.0,
            ),
        ];
        for (
            file,
            source,
            count,
            msg_name,
            msg_id,
            length,
            sig_name,
            start,
            sig_len,
            scale,
            offset,
        ) in cases
        {
            let bus = compile(file, source);
            assert_eq!(bus.messages.len(), count, "{file}");
            let message = bus
                .messages
                .iter()
                .find(|message| message.name == msg_name)
                .unwrap_or_else(|| panic!("{file} missing {msg_name}"));
            assert_eq!(message.id, msg_id, "{file}");
            assert_eq!(message._length, length, "{file}");
            let signal = message
                .signals
                .iter()
                .find(|signal| signal.name == sig_name)
                .unwrap_or_else(|| panic!("{file} missing {sig_name}"));
            assert_eq!(signal.start, start, "{file}");
            assert_eq!(signal.length, sig_len, "{file}");
            assert_eq!(signal.scale, Some(scale), "{file}");
            assert_eq!(signal.offset, Some(offset), "{file}");
        }
    }
}
