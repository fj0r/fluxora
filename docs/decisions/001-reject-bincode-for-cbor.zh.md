# ADR 001: 弃用 Bincode，采用 CBOR

**日期**: 2026-05-08
**状态**: 已采纳
**影响范围**: `message` crate (`ActiveCodec`)、`gateway` crate、所有 codec 相关服务

## 背景

Fluxora 传输层最初选择 bincode 作为默认二进制编码，理由是：
- Rust 生态内性能最优
- 内部 Rust-to-Rust 通信，不需要跨语言兼容

但在实际使用中发现了不可接受的问题，并且逐渐形成了一个更精细的 codec 策略。

## 问题

### 1. serde 内联标签枚举不兼容（根本原因）

Bincode **无法反序列化 `#[serde(tag = "...")]` 或 `#[serde(content = "...")]` 枚举**。它会调用 `deserialize_any`，而 bincode 不支持此方法。这是最主要的技术障碍——Fluxora 的 `Brick` 和 `Content` 类型大量使用内联标签枚举来表达 JSON DSL。虽然这影响所有反序列化场景，但传输层也需要处理这些类型。

### 2. serde 版本不兼容

Bincode 1.x (`1.3.3`) 与 serde 2.x 存在严重兼容性问题。项目已升级到 serde 2.x（`1.0.228+`），bincode 1.x 无法正确序列化/反序列化新版 serde 派生的类型。Bincode 2.x 虽然支持 serde 2.x，但 API 与 1.x **完全不兼容**，且仍处于不稳定状态（RC/alpha），升级成本和维护风险过高。

### 3. 缺乏类型自描述能力

Bincode 是无类型的纯字节流，解码时必须**精确知道目标类型**。这在以下场景成为阻碍：
- **Gateway 消息路由**：Gateway 需要反序列化 envelope 元数据（receiver、event name）来决定路由目标，但 bincode 无法部分解析
- **调试和日志**：二进制流无法直接查看内容，排查问题必须重新编译解码
- **协议演进**：新增字段时必须所有端同步升级，否则直接 panic

### 4. CBOR 覆盖 bincode 的所有优势且更通用

| 维度 | Bincode | CBOR (`ciborium`) |
|------|---------|-------------------|
| 性能 | 最快（零拷贝） | 快（接近 bincode） |
| 跨语言 | 仅 Rust | IETF 标准（RFC 8949） |
| 类型自描述 | 无 | 完整（tag + length prefix） |
| 部分解析 | 不支持 | 支持（offset seeking） |
| serde 2.x | 1.x 不可用，2.x 不稳定 | 稳定 |
| 内联标签枚举 | ❌ 不支持 | ✅ 支持 |
| WASM 兼容 | 部分问题 | 完全兼容 |
| JS 前端 | 无官方库 | 成熟（`cbor-x` 等） |

## 决策

### Codec 策略

**三层 codec 方案，每层策略不同：**

| 层级 | 策略 | 理由 |
|------|------|------|
| **Gateway WS** | 按连接自适应 | 客户端发 Text → JSON，发 Binary → CBOR。无需配置——帧类型本身就是信号。Gateway 记住每个 session 的偏好，用于所有发往该连接的消息（包括来自 Kafka 的 income 推送）。 |
| **Kafka 队列** | 硬编码 CBOR | 全队列流量使用 CBOR。Avro/Schema Registry 作为未来可能性记录——当需要严格 Schema 演进或多语言消费者时考虑。 |
| **UI (WASM)** | 默认 CBOR，查询参数覆盖 | UI 默认发送 CBOR 二进制。如需调试可通过查询参数覆盖。CBOR 是自描述的，无论客户端声明什么，Gateway 总能正确解码。 |

### 为什么不统一配置？

一个全局 codec 配置在纸面上更简单，但实践中行不通：
- **Gateway** 服务异构客户端（调试工具、生产 UI、测试脚本）。自适应检测消除了配置负担。
- **Kafka** 存储持久化数据。固定使用 CBOR——不需要配置。CBOR 的自描述性保证了调试仍然可行，无需切换 JSON。Avro 作为未来选项记录，当出现多语言消费者时考虑。
- **UI** 是用户层。默认应该是最高效的选项（CBOR），并保留调试的逃生口。

## 后果

### 正面
- serde 2.x 兼容性彻底解决
- 内联标签枚举（`#[serde(tag = "...")]`）与 CBOR 正常工作
- Gateway 自动适应客户端 codec——新集成零配置
- Kafka codec 按环境可配置
- CBOR hex dump 可以用在线工具检查

### 负面
- CBOR 比 bincode 大约 10-20%（类型 tag 开销）
- CBOR 编解码比 bincode 慢约 10-15%——对 WS 和 Kafka 工作负载可接受
- 三种不同的 codec 策略增加了概念复杂度（但减少了运维摩擦）

### 迁移
- `gateway.toml` 中 `codec = "bincode"` 需改为 `codec = "cbor"`（或省略，CBOR 已是默认值）
- 已有使用 bincode 编码的在途消息将无法解码（内部系统，可接受一次性断点）

## 为什么 AI 无法做出这个决策

这是一个 AI  consistently 会给出"正确但错误"的决策空间：

- AI 可能建议"为了性能就用 bincode"——技术上正确，但忽略了内联标签枚举的不兼容性。
- AI 可能建议"全部做成可配置的"——架构上整洁，但运维负担重（没人想给每个客户端配 codec）。
- AI 可能建议"用 MessagePack 替代 CBOR"——对比本身有效，但 CBOR 的 IETF 标准化和 JS 生态成熟度在这个上下文中更重要。

正确答案依赖于**代码库之外的知识**：生产 vs 开发工作流的优先级、团队的调试习惯、未来 JS 前端计划、以及对自适应系统优于显式配置的哲学偏好。AI 缺乏这些上下文，会优化不同的轴（性能、简洁性、灵活性），而这些可能与实际需求不符。
