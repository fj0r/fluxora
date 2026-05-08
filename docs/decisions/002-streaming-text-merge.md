# ADR 002: Streaming Text Merge — `Vec<String>` over `&[&str]` and `String`

**Date**: 2026-05-08
**Status**: Accepted
**Scope**: `ui` crate (WASM frontend), stream merge for AI token rendering

## Context

Fluxora's UI renders AI-generated text as a stream of tokens. Each token arrives as a separate WebSocket message containing a small payload embedded in a JSON envelope. The UI must accumulate these fragments and render them incrementally to produce a smooth "typing effect."

Three approaches were considered for storing accumulated fragments.

## Approaches Evaluated

### Option 1: `&[&str]` — Reference Slices

```rust
struct MergeBuffer<'a> {
    fragments: &'a [&'a str],
}
```

**Rejected: lifetime hell + memory amplification.**

Fragments arrive from async channels, network buffers, and parser temporaries — each with different lifetimes. Unifying them into a single `&'a` scope is impractical in an async streaming context.

But there's a deeper problem even if you sidestep lifetimes (e.g. via arena allocators or `Rc`): **references force the original data to stay alive**. A `&str` is just a pointer + length — it doesn't own the bytes. To keep the reference valid, the **full original fragment (envelope + payload) must remain in memory** until rendering completes. You cannot drop the envelope while the payload is still referenced.

This causes memory amplification proportional to the inverse of the payload ratio:

| Payload Ratio | Memory Amplification |
|---------------|---------------------|
| 20% | 5× |
| 10% | 10× |
| 5% | 20× |

If the payload is 20 bytes but the envelope is 80 bytes, every `&str` reference locks 100 bytes in memory. The 80 bytes of envelope are dead weight that cannot be reclaimed.

### Option 2: `String` — Single Growing Buffer

```rust
struct MergeBuffer {
    accumulated: String,
}
// Each new token: accumulated.push_str(&token);
```

**Rejected: allocation thrashing.**

Each `push_str` may trigger a reallocation when capacity is exceeded. For N tokens, this means O(N) potential realloc + memcpy cycles. The buffer grows incrementally, and each reallocation copies all previously accumulated data.

### Option 3: `Vec<String>` — Fragment Buffer

```rust
struct MergeBuffer {
    fragments: Vec<String>,
}
// Each new token: fragments.push(token);
// Render: fragments.iter().flat_map(|s| s.chars()).collect()
```

**Accepted.**

Each fragment owns its memory. New tokens are appended with O(1) `Vec` push — no reallocation of existing data. The final render concatenates all fragments once, performing a single allocation of the known total size.

## Decision

Use `Vec<String>` for fragment accumulation with lazy flattening at render time.

### Why `Vec<String>` > `String`

| Metric | `String` (push_str) | `Vec<String>` (push) |
|--------|---------------------|----------------------|
| Per-token cost | Potential realloc + memcpy of entire buffer | O(1) pointer bump |
| Total allocations | O(N) worst case (one per token) | O(log N) Vec growth + 1 render concat |
| Memory overhead | Minimal | N × 8 bytes (pointer per fragment) |
| Memory content | Identical | Identical |

The `Vec<String>` overhead is negligible — just N pointer slots (8 bytes each on 64-bit). The content stored is the same. The allocation pattern is strictly better: amortized O(1) per token vs potential O(N) realloc per token.

### Render Strategy

The rendering component will eventually be redesigned to consume `Vec<String>` directly, iterating fragments without pre-concatenation. This avoids even the single final allocation. **This is deferred** — the current approach concatenates at render time, which is already a massive improvement over per-token `push_str`.

### Protocol Overhead

Each fragment carries a full JSON envelope (metadata, routing info) where the actual text payload is a small fraction. Storing the full `String` per fragment means memory usage is proportional to the envelope size, not just the payload. This is accepted as the cost of a decoupled architecture — the envelope is needed to locate the correct UI tree node. If this becomes a bottleneck, the optimization is to extract and store only the payload string, discarding the envelope after routing.

## Consequences

### Positive
- Zero lifetime complexity — ownership model handles everything
- Amortized O(1) per-token append — no repeated buffer copying
- Simple to reason about and debug
- Each fragment is independently owned and can be dropped individually

### Negative
- Memory usage proportional to envelope size, not just payload
- Final render requires concatenation (mitigated by future direct `Vec<String>` consumption)

### Future Work
- Redesign text rendering component to consume `Vec<String>` directly without pre-concatenation
- Consider payload-only extraction if envelope overhead becomes a memory bottleneck
