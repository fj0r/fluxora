# ADR 001: Reject Bincode in Favor of CBOR

**Date**: 2026-05-08
**Status**: Accepted
**Scope**: `message` crate (`ActiveCodec`), `gateway` crate, all codec-dependent services

## Context

Fluxora's transport layer initially chose bincode as the default binary encoding based on:
- Best-in-class performance within the Rust ecosystem
- Internal Rust-to-Rust communication — no cross-language compatibility needed

However, practical usage revealed unacceptable drawbacks, and a more nuanced codec strategy emerged.

## Problems

### 1. serde Internally Tagged Enums Incompatibility (Root Cause)

Bincode **cannot deserialize `#[serde(tag = "...")]` or `#[serde(content = "...")]` enums**. It calls `deserialize_any` which bincode does not support. This is the primary technical blocker — Fluxora's `Brick` and `Content` types use internally tagged enums extensively for the JSON DSL. While this affects all deserialization, the wire protocol also needs to handle these types.

### 2. serde Version Incompatibility

Bincode 1.x (`1.3.3`) has severe compatibility issues with serde 2.x. The project has upgraded to serde 2.x (`1.0.228+`), and bincode 1.x cannot correctly serialize/deserialize types derived by the new serde. Bincode 2.x does support serde 2.x, but its API is **fully incompatible** with 1.x and remains in an unstable state (RC/alpha), making the upgrade cost and maintenance risk prohibitive.

### 3. No Type Self-Description

Bincode is an untyped byte stream — decoding requires **exact knowledge of the target type**. This becomes a blocker in the following scenarios:
- **Gateway message routing**: The Gateway must deserialize envelope metadata (receiver, event name) to route messages, but bincode cannot be partially parsed
- **Debugging and logging**: Binary streams cannot be inspected directly — troubleshooting requires recompiling with a decoder
- **Protocol evolution**: Adding fields requires synchronized upgrades across all endpoints, otherwise deserialization panics

### 4. CBOR Covers All of Bincode's Advantages and More

| Dimension | Bincode | CBOR (`ciborium`) |
|-----------|---------|-------------------|
| Performance | Fastest (zero-copy) | Fast (near-bincode) |
| Cross-language | Rust only | IETF standard (RFC 8949) |
| Type self-description | None | Full (tags + length prefixes) |
| Partial parsing | Not supported | Supported (offset seeking) |
| serde 2.x support | 1.x broken, 2.x unstable | Stable |
| Internally tagged enums | ❌ Not supported | ✅ Supported |
| WASM compatibility | Partial issues | Fully compatible |
| JS frontend | No official library | Mature (`cbor-x`, etc.) |

## Decision

### Codec Strategy

**Three-layer codec approach with different policies per layer:**

| Layer | Policy | Rationale |
|-------|--------|-----------|
| **Gateway WS** | Adaptive per-connection | Client sends Text → JSON, Binary → CBOR. No config needed — the frame type is the signal. Gateway remembers the hint per session and uses it for all outgoing messages (including income push from Kafka). |
| **Kafka Queue** | Hardcoded CBOR | CBOR for all queue traffic. Avro/Schema Registry noted as a future possibility if strict schema evolution or polyglot consumers become necessary. |
| **UI (WASM)** | Default CBOR, query-param override | UI defaults to sending CBOR binary. Can override via query parameter if needed for debugging. CBOR is self-describing so the gateway can always decode it regardless of what the client claims. |

### Why Not a Single Configurable Codec Everywhere?

A uniform codec setting is simpler on paper but fails in practice:
- **Gateway** serves heterogeneous clients (debug tools, production UI, test scripts). Adaptive detection removes configuration burden.
- **Kafka** stores durable state. Always CBOR — no config needed. CBOR's self-description means debugging is still possible without switching to JSON. Avro noted as a future option if polyglot consumers arise.
- **UI** is the user-facing layer. Default should be the most efficient option (CBOR), with an escape hatch for debugging.

## Consequences

### Positive
- serde 2.x compatibility fully resolved
- Internally tagged enums (`#[serde(tag = "...")]`) work correctly with CBOR
- Gateway auto-adapts to client codec — zero config for new integrations
- Kafka codec configurable per environment
- CBOR hex dumps can be inspected with standard online tools for debugging

### Negative
- CBOR is ~10-20% larger than bincode due to type tag overhead
- CBOR encode/decode is ~10-15% slower than bincode — acceptable for WS and Kafka workloads
- Three different codec policies increase conceptual complexity (but reduce operational friction)

### Migration
- `gateway.toml` entries with `codec = "bincode"` must be changed to `codec = "cbor"` (or omitted, since CBOR is now the default)
- In-flight messages encoded with bincode will fail to decode (acceptable one-time break for an internal system)

## Why AI Cannot Make This Decision

This is a decision space where AI consistently proposes "correct but wrong" solutions:

- An AI might suggest "just use bincode for performance" — technically correct, but misses the internally tagged enum incompatibility.
- An AI might suggest "make everything configurable" — architecturally clean, but operationally burdensome (nobody wants to configure codec per client).
- An AI might suggest "MessagePack instead of CBOR" — valid comparison, but CBOR's IETF standardization and JS ecosystem maturity matter more in this context.

The right answer depends on knowledge **external to the codebase**: production vs. dev workflow priorities, team debugging habits, future JS frontend plans, and the philosophical preference for adaptive systems over explicit configuration. AI lacks this context and will optimize for different axes (performance, simplicity, flexibility) that may not align with actual needs.
