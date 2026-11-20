# runbot WebSocket Token 认证方案

## 背景

runbot 提供正向 WebSocket 客户端 `loop_client` 和反向 WebSocket 服务端 `loop_server`。此前两个方向都没有读取或发送握手身份凭据：反向服务端在 TCP accept 后直接升级 WebSocket，正向客户端也只连接 URL，因此部署端无法用 runbot 自身配置限制或声明连接身份。

本方案将“接受 token”定义为双向可选的预共享凭据：

- 反向 WS：runbot 作为服务端，在 WebSocket 握手阶段校验客户端发送的 `Authorization: Bearer <access_token>`。
- 正向 WS：runbot 作为客户端，在连接上游时于 WebSocket Upgrade 请求中发送相同形式的 `Authorization` 头。

该方式符合 OneBot 11 反向 WebSocket 常见的 `Authorization: Bearer <access_token>` 约定。未配置 token 时保持原有开放连接行为。

## 目标与非目标

目标是增加可选的入站与出站认证：

- 服务端未配置 token 时不要求 Authorization 头；配置后仅当握手携带匹配凭据时才升级 WebSocket。
- 客户端未配置 token 时不发送 Authorization 头；配置后在每次连接/重连的握手请求中发送 Bearer 凭据。
- 认证失败在握手阶段处理，不创建 `BotContext`、不启动事件处理循环。
- 服务端认证失败不让接入循环退出或 panic；客户端握手失败继续走既有重连等待机制。
- token 不得出现在 URL、日志、错误上下文或 Debug 输出中。

本次不设计 token 刷新、多 token/用户权限、账号绑定、TLS、网络访问控制或连接速率限制。Token 是共享凭据，不代表每个连接的独立身份。

## 建议 API

在现有 builder API 上新增可选配置：

- `BotServerBuilder::access_token(token: impl Into<String>)`：设置服务端要求的 token。
- `BotContextBuilder::access_token(token: impl Into<String>)`：设置客户端连接上游时发送的 token。
- 两个方向未设置时均配置为 `None`，沿用当前公开无认证行为。
- 空 token 在 `build()` 时返回 `ParamsError`，避免产生无法完成认证的配置。
- token 仅保存在内部配置中；日志与 Debug 输出必须脱敏。

`BotServer` 是公开结构体且字段公开，`BotContextBuilder` 也暴露公开字段。为避免 token 成为可直接读取的公开字段，本实现在两者上使用私有字段保存 token：这会破坏直接使用 struct literal 构造 `BotServer` 或 `BotContextBuilder` 的外部源码，但 `BotServerBuilder::new()` / `BotContextBuilder::new()` 链式用法不受影响。`BotContext` 本身已有 `pub(crate)` 字段，外部本来就不能直接使用 struct literal 构造。该兼容性取舍在下面的“兼容性与影响”中说明。

## 握手流程

### 服务端入站（`loop_server`）

1. 接受 TCP 连接后，先读取 HTTP Upgrade 握手请求并校验认证，不立即创建 `BotContext`。
2. 未配置 `access_token`：不要求 Authorization 头，按当前流程升级 WebSocket。
3. 已配置 `access_token`：要求存在有效的 `Authorization: Bearer <token>`。
4. Bearer scheme 按 HTTP 认证语义大小写不敏感；token 值按原样精确匹配，不做 URL 解码、空白修剪或大小写变换。
5. 缺失头、格式错误、scheme 不匹配、空值或 token 不匹配时，拒绝握手并返回 HTTP 401；不进入 `loop_bot`。
6. 认证成功后才完成 WebSocket 升级，随后按现有流程构造连接上下文并处理消息。
7. 单个连接的握手读取或认证错误按单连接错误处理，不终止服务端 accept 循环；后续连接继续接受。

实现使用 tokio-tungstenite 的握手回调能力完成预升级校验。401 响应体只包含通用认证失败信息，不回显收到的凭据。

### 客户端出站（`loop_client`）

1. 每次连接或重连都根据当前 URL 重新构造 WebSocket Upgrade Request。
2. 未配置 `access_token`：不添加 Authorization 头，保持原连接行为。
3. 已配置 `access_token`：在请求头中设置 `Authorization: Bearer <token>`，并将该 header value 标记为敏感；不得把 token 拼入 URL 或 query。
4. 上游返回 401 或其他握手失败时，按既有逻辑等待后重连，不因单次失败退出 `loop_client`。
5. URL 或 header 构造失败属于配置错误，直接返回错误并停止重连；错误信息不包含 token。
6. 握手成功后继续使用现有消息处理、断开和重连语义。

## 失败与可观测行为

- 服务端认证失败属于预期拒绝，不应导致 `loop_server` 主循环退出或 panic。
- 服务端可以记录拒绝事件，但日志仅记录原因类别（如缺失/格式错误/scheme 不匹配/无效），绝不记录完整 Authorization 头、token 或其可逆编码。
- 服务端对外错误统一为 401，避免通过状态码区分 token 缺失和 token 错误。
- 客户端握手失败沿用既有重连等待；日志不得包含 token 或完整 Authorization 头。
- 两个方向都不应在 URL、错误上下文或 Debug 输出中暴露 token。
- 服务端握手读取错误按单连接错误处理；后续连接继续接受。
- 正确 token 连接仍使用现有事件处理和断开/重连语义，不改变 OneBot 消息处理流程。

## 安全与运维要求

- 生产环境应通过 `wss://` 或受信任的 TLS 终止代理保护传输；明文 `ws://` 下 token 可被网络观察者截获。当前 runbot 绑定 TCP 并使用普通 WS，本方案不增加 TLS。
- Token 应由部署方通过安全配置注入，避免硬编码在仓库、日志、示例输出或命令行历史中。文档示例使用环境变量或占位值。
- 服务端认证比较宜采用常量时间比较，避免基于逐字节比较产生的时序差异；是否引入小型比较依赖应在实现阶段根据依赖策略确定。
- 客户端只通过 Authorization 头发送 token，不将凭据置于 URL，避免 URL 日志、代理记录和 Referer 扩散。
- 不应启用 tokio-tungstenite/tungstenite 的握手 TRACE 日志：该依赖会在客户端握手阶段打印完整 HTTP 请求，其中包含 Authorization。runbot 自身日志使用不含凭据的 Display 输出；生产环境应限制日志级别或过滤第三方 handshake target。
- Token 更新需要重启/重建服务端或客户端配置；在线轮换、多凭据并行过渡不在本次范围。
- 防止滥用还需要部署层的网络隔离、连接限流和 TLS；仅增加 token 不应被描述为完整安全边界。

## 兼容性与影响

- 默认不配置 token 时，现有客户端和服务端无需变化，行为保持不变。
- 服务端配置 token 后，旧客户端如果不发送 Bearer 头将无法连接，这是启用认证后的预期行为。
- 客户端配置 token 后，上游必须接受相同 token；未配置认证的上游可能忽略该头或拒绝连接，由部署方保证两端配置一致。
- `BotServer` 和 `BotContextBuilder` 原本暴露公开字段；新增私有 token 字段会破坏使用 struct literal 构造这两个类型的下游源码，即使 builder 链式用法兼容。`BotContext` 本身已有非公开字段，外部原本就不能直接构造。本实现接受该源码兼容影响，并以 builder 作为主要配置入口。
- 需要更新正向/反向 WS 示例和 README：演示两端如何配置 token，并说明默认关闭状态及明文 WS 的凭据泄露风险。

## 验收标准

1. 服务端未设置 token 时，不带 Authorization 的现有反向 WS 客户端仍可连接。
2. 服务端设置 token 后，正确的 Bearer token 可完成握手并进入既有消息处理流程。
3. 服务端设置 token 后，缺失、空白格式错误、错误 scheme、错误 token 的请求均在升级前被拒绝，返回 401，且不创建连接上下文。
4. 单个非法握手不会终止服务端接入循环；后续合法连接仍可成功。
5. 客户端未设置 token 时，握手请求不包含 Authorization 头。
6. 客户端设置 token 后，握手请求包含 `Authorization: Bearer <token>`，且 token 不出现在 URL 中。
7. 客户端遇到 401 或其他握手失败时不会 panic，仍按既有等待逻辑重连；配置构造错误能返回明确错误。
8. 两个方向的日志、错误消息、Debug 输出不包含 token 或完整 Authorization 头。
9. README/示例的配置说明与实际 API 一致，并明确明文 WS 的凭据泄露风险。

## 已确认事项（本次实现）

1. 认证范围同时覆盖 `loop_server` 入站校验和 `loop_client` 出站发送。
2. 接受新增私有字段对 `BotServer` / `BotContextBuilder` struct literal 的源码兼容影响，以 builder API 为主。
3. 未配置 token 时保持现有开放连接行为，避免破坏既有部署。
