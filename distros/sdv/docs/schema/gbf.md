# GBF Schema

A GBF schema is one complete input contract. Its entry file describes the outer packet layout and selects one private payload format. Streams do not combine a GBF packet schema with a separate DBC or ARXML schema.

## Source layout

Every file-backed schema is installed from a ZIP package. The ZIP root must contain exactly one regular entry file. Additional files, when needed, must be placed below the entry's companion directory, whose name is the entry filename without its final extension.

```text
vehicle-gbf.zip
├── vehicle.gbf.json
└── vehicle.gbf/
    └── format/
        ├── 1_powertrain.dbc
        └── 2_body.ldf
```

When a named schema is created, VeloFlux copies this complete source to:

```text
<data_dir>/schemas/gbf/<schema-name>/
```

No archive entry may exist outside the root entry and companion directory. The path supplied to `POST /schemas` identifies the ZIP and is an installation input only. Runtime decoding and restart recovery use the extracted copy. VeloFlux does not discover dependencies beside the archive.

## CAN entry

```json
{
  "signal_name_pattern": "{sig_name}",
  "structure": {
    "type": "struct",
    "fields": [
      { "name": "ts", "type": "u64be" },
      { "name": "total_len", "type": "u16be" },
      {
        "name": "frames",
        "type": "sequence",
        "length_ref": "total_len",
        "length_unit": "bytes",
        "structure": {
          "type": "struct",
          "fields": [
            { "name": "can_id", "type": "u32be" },
            { "name": "data_len", "type": "u8" },
            {
              "name": "payload",
              "type": "bytes",
              "length_ref": "data_len",
              "format": { "id_ref": "can_id" }
            }
          ]
        }
      }
    ]
  },
  "format": {
    "type": "can",
    "props": {
      "dbc_path": "format/vehicle.dbc",
      "clamp_to_range": true,
      "can_id_mapping": "raw"
    }
  }
}
```

`dbc_path` is relative to the companion directory, not to the entry file and not to the process working directory. `can_id_mapping` also accepts `{ "mode": "bus_shift", "bits": 12 }`.

### Separate bus and CAN IDs

For multi-bus inputs, prefer carrying the bus ID and the complete CAN ID in
separate fields. This preserves the full 29-bit CAN ID and avoids allocating a
fixed number of bits to a synthetic packed ID:

```json
{
  "signal_name_pattern": "b{bus_id}_{msg_id_hex_lower}_{sig_name}",
  "structure": {
    "type": "struct",
    "fields": [
      { "name": "ts", "type": "u64be" },
      { "name": "total_len", "type": "u16be" },
      {
        "name": "frames",
        "type": "sequence",
        "length_ref": "total_len",
        "length_unit": "bytes",
        "structure": {
          "type": "struct",
          "fields": [
            { "name": "bus_id", "type": "u8" },
            { "name": "can_id", "type": "u32be" },
            { "name": "data_len", "type": "u8" },
            {
              "name": "payload",
              "type": "bytes",
              "length_ref": "data_len",
              "format": {
                "bus_id_ref": "bus_id",
                "id_ref": "can_id"
              }
            }
          ]
        }
      }
    ]
  },
  "format": {
    "type": "can",
    "props": {
      "dbc_path": "format"
    }
  }
}
```

`bus_id_ref` may reference an earlier `u8`, `u16be`, `u16le`, `u32be`, or
`u32le` field in the same frame structure. When it is present, VeloFlux matches
frames by the structured `(bus_id, can_id)` pair. `can_id_mapping` must be
omitted; configuring both is rejected.

Zero-valued fields are consumed according to the declared frame structure; the
decoder does not treat leading zero bytes as implicit padding. For example, a
CAN-only envelope may declare `{ "name": "frame_type", "type": "u8", "const": 0 }`
as the first frame field.

A root sequence may be followed by fixed-width integer trailer fields. Their
sizes are included when splitting concatenated packets, so a declaration such
as `{ "name": "crc", "type": "u16be" }` consumes a two-byte trailer. GBF does
not currently validate the checksum value. Variable-length trailer fields are
not supported.

For a DBC/LDF directory, filenames use `{bus_id}_{bus_name}.dbc` or
`{bus_id}_{bus_name}.ldf`, for example `1_chassis.dbc` and `2_body.ldf`.
`dbc_path` may point at a single `.dbc` or `.ldf` file, or at a directory of
both. The short pattern above produces SQL-safe names such as
`b1_100_WheelSpeed` without requiring quoted identifiers. It includes the
message ID because signal names need not be unique across messages on the
same bus.

CAN and LIN share `format.type = "can"`. They are the same packed-signal
format. Distinguish buses by the packed frame identity, not by a second GBF
format type.

### LIN identifiers

LDF members compile into the same signal model as DBC. VeloFlux matches an LDF
frame by the 6-bit LIN frame ID from the `Frames` section, for example `0x12`.

GBF does **not** strip a LIN Protected ID (PID). A PID is the 6-bit frame ID
plus two parity bits, so `0x92` is not the same identity as `0x12`. The
collector must put the 6-bit frame ID into `id_ref` (optionally packed with a
bus ID into a `u32`). Parity, if retained, belongs in some other field.

A protected ID on the wire is treated as an unknown frame and dropped.

When several buses share one packed `u32` ID, use `can_id_mapping` bus-shift
(or `bus_id_ref`) so CAN `0x12` and LIN `0x12` on different buses do not
collapse. Raw mapping with two buses that share a frame ID is rejected at
schema install.

## SOME/IP entry

The packet `structure` is defined in the same way. The private format section is:

```json
{
  "signal_name_pattern": "{service}.{method}.{field}",
  "format": {
    "type": "someip",
    "props": {
      "arxml_path": "format/system.arxml"
    }
  }
}
```

The ARXML file belongs only to this GBF resource. It is parsed once while the GBF schema is resolved and is not registered as a nested schema.

`signal_name_pattern` belongs to the complete GBF schema and is therefore an
entry-level property for every private format. It is not read from
`format.props`, schema-install props, decoder props, or merger props.

## Runtime artifact

Resolving the entry produces one logical stream schema and one immutable `CompiledGbfSchema`. The artifact contains the validated packet layout, compiled DBC or ARXML state, naming policy, identity mapping, and decode options. Both the normal decoder and fused merger share it and do not read paths or format options from their own props.

Standalone `dbc` and `arxml` schema types remain available for protocols that use those formats directly. They reuse the same compiler functions but have independent resource lifecycles.
