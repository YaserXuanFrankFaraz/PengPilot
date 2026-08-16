# PengPilot ↔ Waku 差异梳理（0.1.11 同步）

> 分叉点：`bdb4221`（waku 33 个提交 / PengPilot 31 个提交）。本次 0.1.11
> 以"内容移植"方式吸收 waku 共享代码层的优化，保留 PengPilot 单体架构与
> 自有功能。本文档记录两侧的结构差异、已集成项、以及明确不集成的部分。

## 一、架构差异（最重要）

| 维度 | PengPilot（保持） | Waku（现状） |
| --- | --- | --- |
| 进程模型 | 单进程：驱动直接派生 provider CLI，SQLite 进程内 | 桌面端 → `waku-daemon` 子进程（WebSocket RPC）→ `waku-core` 托管会话 |
| 代码组织 | 单体 `src/`（88 个 .rs） | 工作区拆分：`waku-protocol`（线协议类型）、`waku-core`（daemon 实现）、`waku-client`（RPC 适配）、`waku-daemon`（二进制）、`apps/web`（React 客户端） |
| 客户端 | 仅原生 macOS 桌面 | 桌面 + Web（React 19 + TanStack）多客户端共享会话 |
| 网络面 | 零（无 socket、无 token、无端口） | 本地回环 + 可选 LAN 暴露（token / origin 白名单） |
| 会话生命周期 | 随 App 退出终止 | daemon 常驻，会话跨 App 重启存活 |
| 遥测 | 无（0.1.6 起移除） | `analytics.rs`（rust-umami，可选） |

**结论**：daemon 架构是 waku 为多客户端（Web/未来移动端）做的产品选择，不是
性能优化；所有性能优化（120ms 批处理、veil、脉冲时钟、缓存岛）都在 UI 层，
两种架构通用。PengPilot 的移动端路线（Android/iOS）若落地，daemon 拆分应作
为独立项目重开，届时可复用 waku 的协议层设计。

## 二、已集成（0.1.11）

### 流式渲染性能
- `src/app/streaming.rs`：`pop_stream_chunk`（逐帧字素预算）→ `pop_stream_batch`
  （同种 delta 整批合并，全文一次进布局）；`RuntimeEventCursorAdvanced` /
  `UserInputRequested` 事件处理
- `src/app/runtime.rs`：ReasoningDelta 计入 markdown_changed（推理流路由到
  合并的 120ms StreamFrame 节奏，避免 40+ 次/秒全量重渲染）；120ms 帧间隔
- `src/md/veil.rs`（新）：流式文本 paint-only 渐隐（VEIL_EMA 160ms →
  120–400ms 时长，pow 1.6 曲线），文本立即进布局、只动颜色
- `src/md/render.rs`：veil 集成、`markdown_tail`（推理窗口尾段受限渲染
  O(window)）、代码块复制反馈、`BLOCK_ORDINAL_STRIDE_BITS` 稳定键
- `src/ui/motion.rs`（新）：共享脉冲时钟（33ms tick ≈30fps、lease 自动停放、
  reduce-motion 归零）；`pulse()` / `spin()` 元素
- `src/app/render.rs` + `src/app.rs`：WakuPane 缓存岛（sidebar/transcript/
  right-panel 各自 cached，脉动只脏化所属岛）
- `src/app/transcript_view.rs`：锚点尾距保持、活动滚动遏制与 fade、多字节
  内容滑动安全（char boundary）、导航轨 fade
- `src/app/tests.rs` / `src/app/components.rs`：对应测试与组件更新

### 交互与功能
- **Agent 提问**：`UserInputQuestion`/`UserInputAnswer` 类型 + 提问卡片 UI
  （选项/多选/自定义答案/前进后退）+ `DriverControl::respond_user_input`
- **Claude 上下文窗口**：`ProviderModel.context_windows`、`AgentSession.context_window`、
  `PersistedState.last_context_window` + `remembered_model_traits` 扩展 + 模型选择器
- **Codex `/fast`**：`composer_complete::is_fast_mode_toggle_submission` /
  `toggled_fast_service_tier` + builtin 命令
- **队列消息逐条 steer**：`steer_queued_message`
- **模型目录刷新**：模型选择器打开时 `refresh_provider_model_discovery`
- **窗口状态恢复**：`PersistedWindowState`（x/y/width/height/maximized/display
  UUID，Zed 方案）
- **终端 overlay 滚动条** + 字体测量（`src/terminal.rs` 整体移植）
- **滚动遏制** `ui::contain_scroll`；**命令图标**（slash → command）；**复制
  反馈**；composer 高度封顶；拖选窗口级跟踪

### 修复
- 用户气泡引号/列表塌缩、推理窗口滑动 char-boundary panic、Markdown 消息
  宽度、未加载历史误渲染新任务提示、wheel 滚动穿透嵌套滚动区

### 本地化适配（daemon 调用 → 本地实现）
- `prefetch_checkpoint_refs` → `checkpoint::session_turn_refs`
- branches/commit_dialog → `git_branch::*` / `git_commit::*`
- usage_page 用量扫描 → `usage_history::scan`（共享 rate table 缓存）
- usage_meter → 本地 fetcher + `provider_binary_overrides`
- skills_page 扫描/开关/删除 → `skills::*` + `platform::trash_item`
- composer 附件 → 本地路径 + blob store（保留预览图能力）
- 附件图片 → 本地 `blob_store::shared_path_for`
- 全量移除 `crate::analytics` 引用（保持无遥测）

## 三、明确不集成（记为差异）

| 项 | 原因 |
| --- | --- |
| daemon + WebSocket 协议 + `waku-protocol` 线层 | 架构决策差异（见上）；移动端落地时重开 |
| `apps/web` React 客户端 + `packages/waku-client` TS | 同上 |
| `analytics.rs`（rust-umami） | PengPilot 产品决策：无遥测（CHANGELOG 0.1.6） |
| `review_diff.rs` daemon 化重构 | 上游改为接收 daemon 捕获的 git 输出；PengPilot 保留本地捕获，无 UI 差异 |
| 驱动层 `respond_user_input` 真实实现（claude/codex/deepseek/acp/opencode） | 需要真实 CLI 端到端验证（AGENTS.md 发布门槛）；当前为 trait 默认 no-op，后续按 provider 逐个验证移植 |
| Claude 上下文窗口的驱动侧（model-id 后缀） | 同上，需真实会话验证 |

## 四、文件级差异规模（当前状态）

共享文件已对齐到上游内容（veil/motion/scrollbar/transcript_view/components/
streaming/render/runtime/composer/sessions/settings/sidebar/usage_page/
terminal/input/md 全部同步）；仍保留 PengPilot 独有的：

- `src/work.rs`、`src/app/inbox.rs`：Inbox/看板（任务管理）
- `src/driver/json_cli.rs`、`deepseek.rs`/`deepseek_session.rs`/`deepseek_pool.rs`
  等扩展 provider 驱动（Kiro/Hermes/Omp/Prime/Kimi/Antigravity/CodeBuddy/
  Copilot/DevEco/Qoder/Qwen/QwenPaw/Reasonix/Trae）
- `src/model.rs` `ProviderKind` 保留 14 个活动 + 24 个 legacy 变体（waku 仅 8 个）

## 五、后续路线建议

1. 驱动层逐步补 `respond_user_input` / 上下文窗口真实实现（每个 provider
   一次真实 CLI 验证后合入）
2. 移动端立项时：以 waku-protocol 为蓝本设计 PengPilot 协议层，桌面端先
   拆 `waku-core` 等价物（会话/持久化独立于 UI），Web/PWA 客户端可复用
   waku 的 `apps/web` 思路
3. 发布基建：补齐 .env（Apple 公证）、rclone R2 配置、Sparkle 私钥后，
   可走正式 `bun run release` 全流程
