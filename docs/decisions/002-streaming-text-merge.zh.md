# ADR 002: 流式文本合并 — 选择 `Vec<String>` 而非 `&[&str]` 或 `String`

**Date**: 2026-05-08
**Status**: Accepted
**Scope**: `ui` crate（WASM 前端），AI token 渲染的流式合并

## Context

Fluxora 的 UI 以流式方式渲染 AI 生成的文本。每个 token 作为独立的 WebSocket 消息到达，消息中包含一个嵌入在 JSON 信封中的小型 payload。UI 必须累积这些片段并逐步渲染，以产生流畅的"打字效果"。

三种方案被评估用于存储累积的片段。

## 评估方案

### 方案 1: `&[&str]` — 引用切片

```rust
struct MergeBuffer<'a> {
    fragments: &'a [&'a str],
}
```

**已拒绝：生命周期地狱 + 内存放大。**

片段来自异步通道、网络缓冲区和解析器临时变量——各自拥有不同的生命周期。将它们统一到单个 `&'a` 作用域在异步流式场景中几乎不可行。

但即使绕过生命周期问题（比如用 arena allocator 或 `Rc`），还有一个更深层的问题：**引用会强制原始数据一直存活**。`&str` 只是指针 + 长度——它不拥有数据。为了保持引用有效，**完整的原始片段（信封 + payload）必须一直驻留内存**，直到渲染完成。你不能在 payload 还被引用时丢弃信封。

这导致内存放大系数为 payload 占比的倒数：

| Payload 占比 | 内存放大倍数 |
|-------------|------------|
| 20% | 5× |
| 10% | 10× |
| 5% | 20× |

如果 payload 是 20 字节但信封是 80 字节，每个 `&str` 引用都会锁定 100 字节在内存中。那 80 字节的信封是纯开销，无法回收。

### 方案 2: `String` — 单一增长缓冲区

```rust
struct MergeBuffer {
    accumulated: String,
}
// 每个新 token: accumulated.push_str(&token);
```

**已拒绝：分配抖动。**

每次 `push_str` 在容量不足时可能触发重新分配。对于 N 个 token，这意味着 O(N) 次潜在的 realloc + memcpy 循环。缓冲区逐步增长，每次重新分配都要复制之前累积的所有数据。

### 方案 3: `Vec<String>` — 片段缓冲区

```rust
struct MergeBuffer {
    fragments: Vec<String>,
}
// 每个新 token: fragments.push(token);
// 渲染: fragments.iter().flat_map(|s| s.chars()).collect()
```

**已接受。**

每个片段拥有自己的内存。新 token 以 O(1) 的 `Vec` push 追加——不重新分配已有数据。最终渲染时一次性拼接所有片段，只需一次已知总大小的分配。

## 决策

使用 `Vec<String>` 进行片段累积，在渲染时惰性扁平化。

### 为什么 `Vec<String>` > `String`

| 指标 | `String` (push_str) | `Vec<String>` (push) |
|------|---------------------|----------------------|
| 单 token 开销 | 可能 realloc + 拷贝整个缓冲区 | O(1) 指针移动 |
| 总分配次数 | O(N) 最坏情况（每个 token 一次） | O(log N) Vec 增长 + 1 次渲染拼接 |
| 内存开销 | 极小 | N × 8 字节（每个片段一个指针） |
| 内存内容 | 相同 | 相同 |

`Vec<String>` 的开销可以忽略不计——只是 N 个指针槽位（64 位上每个 8 字节）。存储的内容是一样的。分配模式严格更优：每个 token 摊销 O(1) vs 每个 token 可能 O(N) 的 realloc。

### 渲染策略

渲染组件最终将被重新设计为直接消费 `Vec<String>`，无需预拼接即可迭代片段。这甚至避免了最终那一次分配。**此为 deferred**——当前方案在渲染时拼接，但相比每 token `push_str` 已经是巨大的改进。

### 协议开销

每个片段携带完整的 JSON 信封（元数据、路由信息），而实际文本 payload 只占很小一部分。存储完整的 `String` 意味着内存使用与信封大小成正比，而不仅是 payload。这是解耦架构的已接受成本——信封用于定位正确的 UI 树节点。如果这成为瓶颈，优化方案是仅提取并存储 payload 字符串，路由后丢弃信封。

## Consequences

### 正面
- 零生命周期复杂度——所有权模型处理一切
- 每 token 摊销 O(1) 追加——无重复缓冲区拷贝
- 易于推理和调试
- 每个片段独立拥有，可以单独释放

### 负面
- 内存使用与信封大小成正比，不仅是 payload
- 最终渲染需要拼接（通过未来直接消费 `Vec<String>` 缓解）

### 未来工作
- 重新设计文本渲染组件，直接消费 `Vec<String>` 而无需预拼接
- 如果信封开销成为内存瓶颈，考虑仅提取 payload
