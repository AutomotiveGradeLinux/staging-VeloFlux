# BusMirror Schema

A `busmirror` schema is one complete AUTOSAR Bus Mirroring v1 contract. Its JSON
entry defines the network topology, private DBC or LDF files, and output signal
names. Those files are private members of this schema; they are not separately
registered schemas.

## Source package

Install the schema from a server-local ZIP through `POST /schemas`:

```text
vehicle-busmirror.zip
├── vehicle.json
└── vehicle/
    ├── powertrain.dbc
    └── body.ldf
```

The ZIP root contains exactly one entry file. All members must be regular
`.dbc` or `.ldf` files below the entry's same-stem companion directory. Absolute
paths, parent traversal, backslashes, and files outside that directory are
rejected.

After installation, VeloFlux restores the schema from:

```text
<data_dir>/schemas/busmirror/<schema-name>/
```

The original ZIP path is not used at runtime or after restart.

## Entry format

```json
{
  "version": "v1",
  "signal_name_pattern": "{network_type}{network_id}__{msg_id_hex_lower}__{sig_name}",
  "buses": [
    {
      "network_type": "can",
      "network_id": 1,
      "name": "Powertrain",
      "source_file": "powertrain.dbc"
    },
    {
      "network_type": "lin",
      "network_id": 2,
      "name": "Body",
      "source_file": "body.ldf"
    }
  ]
}
```

`version` must be `v1`. There is no `format` block: the outer protocol is
BusMirror, and each bus selects its network with `network_type`. `network_type`
may be `can` or `lin`. Classic CAN and CAN FD use the same CAN network
declaration. Each `(network_type, network_id)` pair and each frame identity
within a bus must be unique.

`source_file` is the private DBC or LDF member for that bus, relative to the
companion directory:

| `network_type` | Allowed `source_file` |
|---|---|
| `can` | `.dbc` |
| `lin` | `.ldf` or `.dbc` |

A CAN bus with an LDF is rejected. A LIN bus may still use a DBC when the
signals were exported that way.

Unknown JSON fields, including a leftover `format` object or a `dbc` member
path, are rejected.

## LDF members

LDF files are compiled into the same signal model as DBC. VeloFlux parses LIN
2.0 through 2.2A, skips unknown top-level sections, and compiles unconditional
numeric frames only.

Event-triggered, sporadic, and diagnostic frames are not decoded. An
event-triggered PID on the wire is treated as an unknown frame and dropped.
ASCII/BCD encodings and non-linear `physical_value` ranges fail schema
installation.

LIN identifiers are normalized to 6 bits. A protected ID such as `0x92` matches
LDF frame ID `0x12`. Schema installation fails when two LIN definitions collapse
to the same normalized identity.

## Signal names

`signal_name_pattern` is compiled when the schema is installed. The default is:

```text
{bus_name}_{msg_name}_{sig_name}
```

BusMirror DBC/LDF tokens are:

| Token | Value |
|---|---|
| `{bus_name}` | Bus name from the entry |
| `{msg_id}` | Normalized message ID in decimal |
| `{msg_id_hex_lower}` / `{msg_id_hex_upper}` | Normalized message ID in hexadecimal |
| `{msg_name}` | Message / frame name |
| `{sig_name}` | Signal name |

BusMirror network tokens are:

| Token | Value |
|---|---|
| `{network_type}` | `can` or `lin` |
| `{network_type_id}` | Numeric AUTOSAR network type in decimal |
| `{network_type_id_hex_lower}` / `{network_type_id_hex_upper}` | Numeric network type in hexadecimal |
| `{network_id}` | Network ID in decimal |
| `{network_id_hex_lower}` / `{network_id_hex_upper}` | Network ID in hexadecimal |

BusMirror has no public bus ID. The DBC compiler's packed internal lookup key is
not part of the schema contract, so `{bus_id}`, `{bus_id_hex_lower}`, and
`{bus_id_hex_upper}` are rejected for BusMirror patterns.

Hexadecimal tokens do not add a prefix. Add a literal prefix when needed, for
example `0x{msg_id_hex_upper}`. Unknown, empty, or unclosed tokens, empty output
names, duplicate output names, and a generated `ts` name are rejected during
installation. Legacy aliases such as `{id}` and `{sig}` are not accepted.

CAN wire flags for extended frames and CAN FD are not part of the DBC identity.
The decoder uses the normalized 29-bit numeric CAN ID emitted by VeloFlux's DBC
parser. LIN uses the 6-bit frame ID. Network type and network ID remain part of
the BusMirror frame identity, so equal IDs on different buses do not collide.
