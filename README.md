# VeloFlux

> Synchronizing logic with motion.

VeloFlux is an open-source, high-performance stream processor for
resource-sensitive automotive environments. It connects vehicle data sources
with SQL-based processing, runtime observability, and edge-oriented sinks.

## Highlights

- Automotive data support, including AUTOSAR Bus Mirroring, CAN/CAN FD, LIN,
  DBC and LDF schemas, GBF, SOME/IP, and Vehicle Signal Specification
  integrations.
- SQL-based filtering, projection, aggregation, stateful functions, and event
  time windows.
- MQTT, file, HTTP, memory, NNG, Kura, and video connectors.
- A Rust implementation designed for predictable resource use and low
  processing overhead.
- Prometheus metrics, profiling, audit logging, and pipeline diagnostics.

## Quick Start

### Prerequisites

- A stable Rust toolchain.
- `make`.
- `protoc`, the protobuf compiler.
- An MQTT broker for the examples below.
- `zip` and [MQTTX CLI](https://mqttx.app/downloads) for the BusMirror
  walkthrough.

On Debian or Ubuntu, the required system packages can be installed with:

```bash
sudo apt-get update
sudo apt-get install -y make protobuf-compiler zip
```

Install MQTTX CLI from its downloads page or, with Node.js 18 or later, through
npm:

```bash
npm install -g mqttx-cli
```

### Build and run

```bash
make build
cargo run --bin veloflux -- --data-dir ./tmp/data
```

The manager API listens on `http://127.0.0.1:8080` by default. Runtime
configuration can be loaded with `--config`; see
[`etc/config.example.yaml`](etc/config.example.yaml) for the available fields.

## Automotive Walkthrough: BusMirror over MQTT

This walkthrough uses the repository's tested AUTOSAR Bus Mirroring v1 fixture.
VeloFlux receives BusMirror destination frames over MQTT, decodes their CAN
and LIN payloads with private DBC or LDF files, and publishes selected signals
as JSON.

### 1. Build a BusMirror schema package

A BusMirror schema is installed as a ZIP containing one JSON entry and its
same-stem companion directory of DBC or LDF files:

```text
vehicle-busmirror.zip
├── vehicle.json
└── vehicle/
    ├── propulsion.dbc
    └── chassis.dbc
```

Build the sample package from the repository fixtures:

```bash
cd distros/sdv/tests/data/busmirror
zip -r /tmp/vehicle-busmirror.zip vehicle.json vehicle
cd -
```

The sample schema names signals with the pattern
`{network_type}{network_id}__{msg_id_hex_lower}__{sig_name}`. For example,
the `RPM` signal on CAN network 1 and frame `0x100` becomes
`can1__100__RPM`.

### 2. Install the schema

The schema source path is read when the schema is installed. VeloFlux copies
and validates the complete artifact under its data directory.

```bash
curl -s -XPOST http://127.0.0.1:8080/schemas \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "vehicle_busmirror",
    "revision": 1,
    "type": "busmirror",
    "props": {
      "schema_path": "/tmp/vehicle-busmirror.zip"
    }
  }' | jq .
```

### 3. Create the BusMirror input stream

The stream references the installed schema. All topology, DBC, and signal-name
configuration comes from that schema, so the decoder needs no repeated paths or
format properties.

```bash
curl -s -XPOST http://127.0.0.1:8080/streams \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "vehicle_input",
    "revision": 1,
    "type": "mqtt",
    "schema": { "ref": "vehicle_busmirror" },
    "props": {
      "broker_url": "tcp://127.0.0.1:1883",
      "topic": "vehicle/busmirror",
      "qos": 0
    },
    "shared": false,
    "decoder": { "type": "busmirror", "props": {} }
  }' | jq .
```

Describe the stream to inspect the columns compiled from the DBC files:

```bash
curl -s http://127.0.0.1:8080/streams/describe/vehicle_input | jq .spec.schema.columns
```

### 4. Create and start a pipeline

The following pipeline publishes a representative signal from each sample CAN
network:

```bash
curl -s -XPOST http://127.0.0.1:8080/pipelines \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "busmirror_pipeline",
    "revision": 1,
    "sql": "SELECT ts, `can1__100__RPM` AS rpm, `can2__200__FL` AS front_left_speed FROM vehicle_input",
    "sinks": [
      {
        "id": "processed_mqtt",
        "type": "mqtt",
        "props": {
          "broker_url": "tcp://127.0.0.1:1883",
          "topic": "vehicle/processed",
          "qos": 0
        },
        "encoder": { "type": "json", "props": {} }
      }
    ]
  }' | jq .

curl -s -XPOST \
  http://127.0.0.1:8080/pipelines/busmirror_pipeline/start
```

Subscribe to `vehicle/processed`, then publish a tested BusMirror payload:

```bash
mqttx sub -h 127.0.0.1 -p 1883 -t vehicle/processed
```

In another terminal:

```bash
mqttx pub -h 127.0.0.1 -p 1883 \
  -t vehicle/busmirror \
  -m '01000000000003e80007a1200022000061010000010008dc05325f000000000005610200000200089411f8113011c611' \
  -f hex
```

Each valid BusMirror destination frame produces one decoded row. A single MQTT
payload may contain multiple destination frames and therefore produce multiple
rows.

For the complete schema contract, decoder semantics, and optional fused packer
configuration, see:

- [BusMirror schema](distros/sdv/docs/schema/busmirror.md)
- [BusMirror decoder](distros/sdv/docs/decoder/busmirror.md)

## Yoriito VISS via Kura

VeloFlux includes a built-in `kura` sink for publishing pipeline columns to a
Kura server through the Yoriito VISS v1 producer API. The connector maps output
column names to VSS paths and sends one unary gRPC `SetCurrent` request for each
output collection.

Create a JSON mapping file whose keys exactly match the pipeline's output
columns. A string value preserves the veloFlux value type. Use an object with
`data_type` when the target VSS node requires `float32` or `float64`:

```json
{
  "speed": {
    "path": "Vehicle.Speed",
    "data_type": "float32"
  },
  "vin": "Vehicle.VehicleIdentification.VIN"
}
```

For example, save the following mapping for the BusMirror stream created above:

```bash
sudo install -d /etc/veloflux
sudo tee /etc/veloflux/kura_mapping.json >/dev/null <<'EOF'
{
  "speed": {
    "path": "Vehicle.Speed",
    "data_type": "float32"
  }
}
EOF
```

Create and start a second pipeline that sends the aliased BusMirror signal to
Kura. The `float32` mapping converts the DBC-derived Float64 value to the VISS
`float` required by `Vehicle.Speed`:

```bash
curl -s -XPOST http://127.0.0.1:8080/pipelines \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "busmirror_kura_pipeline",
    "revision": 1,
    "sql": "SELECT `can2__200__FL` AS speed FROM vehicle_input",
    "sinks": [
      {
        "id": "viss_output",
        "type": "kura",
        "props": {
          "addr": "http://127.0.0.1:50053",
          "mapping_path": "/etc/veloflux/kura_mapping.json"
        }
      }
    ]
  }' | jq .

curl -s -XPOST \
  http://127.0.0.1:8080/pipelines/busmirror_kura_pipeline/start
```

Publish the same tested BusMirror payload:

```bash
mqttx pub -h 127.0.0.1 -p 1883 \
  -t vehicle/busmirror \
  -m '01000000000003e80007a1200022000061010000010008dc05325f000000000005610200000200089411f8113011c611' \
  -f hex
```

Read the current value through the Yoriito consumer API. When running from the
repository checkout, `grpcurl` can load the bundled protobuf definition:

```bash
grpcurl -plaintext \
  -import-path src/flow/proto \
  -proto yoriito/viss/v1/consumer.proto \
  -d '{"path":"Vehicle.Speed"}' \
  127.0.0.1:50052 \
  yoriito.viss.v1.consumer.VISS/GetCurrent
```

The tested payload leaves `Vehicle.Speed` at `45` with VISS type `float`. The
manager automatically uses `encoder=none` for this sink. Boolean, integer,
floating-point, and string values use their native VISS types unless a numeric
mapping override is present. Null values and individually unmapped columns are
skipped; a mapping that matches none of the pipeline output columns is an
error. The current integration is output-only and calls `SetCurrent`; Yoriito
consumer operations and the other producer RPCs are not exposed by veloFlux.

See the [Kura sink documentation](docs/runtime/sinks/connectors/kura.md) for
batching behavior, value conversion, and current limitations.

## History Query

VeloFlux supports querying historical Parquet files through the **table** catalog
concept. Unlike a stream that continuously subscribes to new data, a table is a
finite data source — it scans all matching Parquet files once and then ends the
pipeline's data path.

### Prerequisites

Parquet files must follow the NanoMQ history naming convention:

```text
nanomq_{topic}-{start_ts}~{end_ts}_{seq}_{hash}.parquet
```

For example: `nanomq_can0-1700000000~1700001000_42_abc123.parquet`

Each Parquet row must contain:

- A time column (default `ts`) as `Int64` or `UInt64`.
- A `data` column as `Binary` holding the same raw AUTOSAR BusMirror payload
  accepted by the `vehicle_input` MQTT stream above.

### Create a History Table

Register a table definition via the manager API. The `schema` declares the
decoded row structure, `props` points to the Parquet data directory, and
`decoder` tells the runtime how to deserialize the `data` column bytes.

```bash
curl -s -XPOST http://127.0.0.1:8080/tables \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "vehicle_history",
    "revision": 1,
    "type": "history",
    "schema": { "ref": "vehicle_busmirror" },
    "props": {
      "datasource": "/var/lib/nanomq/history",
      "topic": "can0",
      "time_column": "ts",
      "batch_size": 100
    },
    "decoder": {
      "type": "busmirror",
      "props": {}
    }
  }' | jq .
```

Field details:

- **schema.ref**: reuses the `vehicle_busmirror` schema installed earlier, so
  the stream and table share the same DBC files, topology, and output columns.
- **props.datasource**: directory containing the Parquet files.
- **props.topic**: used to discover matching files by filename prefix. Here,
  `can0` matches files beginning with `nanomq_can0-`.
- **props.time_column**: Parquet column containing each row's timestamp
  (default `ts`). Files are processed by filename sequence number, and rows
  retain their Parquet read order.
- **props.batch_size**: rows read per Parquet batch (default `100`).
- **decoder.type**: uses the same `busmirror` decoder as the MQTT stream. The
  decoder receives the raw `data` bytes from each Parquet row and produces the
  DBC-derived columns from `vehicle_busmirror`.

The Parquet time column is metadata used while reading history files and is not
automatically added to the decoded row. The SQL-visible `ts` column is produced
by the BusMirror decoder from each destination-frame header, just as it is for
the MQTT stream.

### Query the Table

Once registered, use the table name in a SQL `FROM` clause:

```sql
SELECT * FROM vehicle_history
```

Scalar projection and filtering are supported:

```sql
SELECT `can2__200__FL` AS speed FROM vehicle_history WHERE ts > 0
```

Current table-scan limitations:

- Table aliases are not supported.
- Stream-table joins are not supported.
- Aggregate queries require `GROUP BY eoswindow()`.

### Send Query Results to Kura

Reuse `/etc/veloflux/kura_mapping.json` created in the Kura walkthrough above.
Both pipelines alias `can2__200__FL` to `speed`, so they have the same final
output column and VSS mapping.

Create and start a pipeline that scans the table once and sends each decoded
speed value to Kura:

```bash
curl -s -XPOST http://127.0.0.1:8080/pipelines \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "vehicle_history_kura_pipeline",
    "revision": 1,
    "sql": "SELECT `can2__200__FL` AS speed FROM vehicle_history",
    "sinks": [
      {
        "id": "viss_output",
        "type": "kura",
        "props": {
          "addr": "http://127.0.0.1:50053",
          "mapping_path": "/etc/veloflux/kura_mapping.json"
        }
      }
    ]
  }' | jq .

curl -s -XPOST \
  http://127.0.0.1:8080/pipelines/vehicle_history_kura_pipeline/start
```

The manager automatically uses `encoder=none` for the Kura sink. The mapping
key must exactly match the final SQL output column name.

### Diagnostics

Inspect the registered table metadata and the pipeline execution plan:

```bash
curl -s http://127.0.0.1:8080/tables \
  | jq '.[] | select(.name == "vehicle_history")'
curl -s \
  http://127.0.0.1:8080/pipelines/vehicle_history_kura_pipeline/explain
```

## Pipeline Diagnostics

Inspect the pipeline's desired state, execution plan, and runtime processor
statistics:

```bash
curl -s http://127.0.0.1:8080/pipelines/busmirror_pipeline | jq .status
curl -s http://127.0.0.1:8080/pipelines/busmirror_pipeline/explain
curl -s http://127.0.0.1:8080/pipelines/busmirror_pipeline/stats | jq .
```

## Project Layout

- `src/parser/` parses the SQL dialect into the `SelectStmt` contract.
- `src/flow/` contains planning, processing, connectors, and the runtime.
- `src/datatypes/` contains shared schemas and runtime values.
- `src/manager/` provides the manager API and resource lifecycle.
- `distros/sdv/` provides automotive schemas, codecs, and integrations.
- `docs/` contains design, syntax, runtime, API, and integration documentation.

## Development

```bash
make fmt
make clippy
make test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a change.

## License

VeloFlux is licensed under the [Mozilla Public License 2.0](LICENSE), except for
components that declare a different license in their own package metadata or
source headers.
