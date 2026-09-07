// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Pluggable payload decoder abstraction for GBF frames.
//!
//! The GBF transport layer extracts `(timestamp, optional_bus_id, format_id,
//! payload)` tuples from binary packets. This module defines the trait that
//! payload decoders implement to convert those tuples into structured output
//! rows.

use flow::model::Tuple;
use flow::planner::decode_projection::DecodeProjection;

/// AUTOSAR BusMirror network type for classic CAN and CAN FD.
pub const BUSMIRROR_NETWORK_CAN: u8 = 1;
/// AUTOSAR BusMirror network type for LIN.
pub const BUSMIRROR_NETWORK_LIN: u8 = 2;
const BUSMIRROR_CAN_FRAME_ID_MASK: u32 = 0x1fff_ffff;
const BUSMIRROR_LIN_FRAME_ID_MASK: u32 = 0x3f;

/// Protocol-normalized DBC frame identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FrameIdentity(u64);

impl FrameIdentity {
    /// Promote a GBF/CAN format ID into the shared identity space.
    #[inline]
    pub const fn gbf(format_id: u32) -> Self {
        Self(format_id as u64)
    }

    /// Preserve separate bus and CAN IDs without sacrificing CAN-ID bits.
    #[inline]
    pub const fn gbf_bus(bus_id: u32, format_id: u32) -> Self {
        Self(((bus_id as u64) << 32) | format_id as u64)
    }

    /// Build an AUTOSAR BusMirror identity.
    ///
    /// LIN frame IDs are normalized to the 6-bit identifier so a protected ID
    /// on the wire matches the frame ID stored in an LDF.
    #[inline]
    pub const fn busmirror(network_type: u8, network_id: u8, frame_id: u32) -> Self {
        let frame_id = if network_type == BUSMIRROR_NETWORK_LIN {
            frame_id & BUSMIRROR_LIN_FRAME_ID_MASK
        } else {
            frame_id & BUSMIRROR_CAN_FRAME_ID_MASK
        };
        Self(((network_type as u64) << 40) | ((network_id as u64) << 32) | (frame_id as u64))
    }

    /// Build a BusMirror identity from the schema compiler's packed bus ID.
    #[inline]
    pub const fn busmirror_bus(bus_id: u32, frame_id: u32) -> Self {
        Self::busmirror((bus_id >> 8) as u8, bus_id as u8, frame_id)
    }

    #[inline]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// A decoded frame ready for payload-level decoding.
#[derive(Debug, Clone)]
pub struct GbfPayloadFrame<'a> {
    /// Packet timestamp.
    pub timestamp: u64,
    /// Separate CAN bus ID when the GBF schema configures `bus_id_ref`.
    pub bus_id: Option<u32>,
    /// Message / format identifier (CAN ID or SOME/IP message ID).
    pub format_id: u32,
    /// Raw payload bytes (borrowed from the transport buffer).
    pub payload: &'a [u8],
}

/// Trait for pluggable GBF payload decoders.
///
/// Implementations handle CAN (DBC), SOME/IP (ARXML), and future formats.
pub trait PayloadDecoder: Send + Sync {
    /// Return `true` when this decoder recognises `format_id`.
    fn contains_format_id(&self, format_id: u32) -> bool;

    /// Decode a batch of frames into a single [`Tuple`].
    ///
    /// Returns `None` when no frame produced any matching output.
    fn decode_frames(
        &self,
        frames: Vec<GbfPayloadFrame<'_>>,
        projection: Option<&DecodeProjection>,
    ) -> Option<Tuple>;
}
