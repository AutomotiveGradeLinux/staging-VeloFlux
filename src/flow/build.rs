// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate kura (yoriito VISS) gRPC client from proto definitions.
    // We only build the client stubs — the server side is not needed.
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["proto/yoriito/viss/v1/producer.proto"], &["proto"])?;

    // Generate test-only proto message for decoder/integration tests.
    prost_build::Config::new()
        .out_dir(std::env::var("OUT_DIR").unwrap())
        .compile_protos(&["proto/test/simple.proto"], &["proto/test"])?;
    Ok(())
}
