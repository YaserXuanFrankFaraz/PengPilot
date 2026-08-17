# Changelog

All notable changes to PengPilot. This file is the **source of truth for the release
notes shown in the in-app updater**: [`scripts/release.ts`](scripts/release.ts)
extracts the section whose heading matches the version being released
(`Cargo.toml`) and publishes it next to the update, so Sparkle shows it in
the update prompt.

Format follows [Keep a Changelog](https://keepachangelog.com). Add a new
`## [<version>]` section at the top for each release, matching the version in
`Cargo.toml`.

Write release notes for the final product users receive, not the development
history. When a feature is still unreleased, fold its fixes and refinements into
the original feature bullet instead of adding separate entries for them.

## [0.1.16]

### Media library and Grok Imagine

- Generated Grok Imagine images show inline in the transcript (including final
  replies), open in the image preview on click, and can be saved manually into a
  **Media Library** when you like them. Saves copy files into owned app storage
  so they survive Grok session cleanup.
- Media Library lives on the left nav rail (below All Tasks and Quadrants):
  adaptive waterfall grid, newest/oldest sort, click-for-detail, right-click to
  reveal or delete. Tags, favorites, and non-image kinds stay out of this build.

### Global chrome titlebar

- The home shell now uses a full-width top chrome bar for the macOS traffic
  lights, window dragging, sidebar toggle, back/forward, and right-panel
  toggle. Column borders start below that bar, so the traffic lights are no
  longer bisected by the rail/list divider.

### Provider verification (ad-hoc dogfooding release)

| Provider | CLI version | Test command | Result | Date |
| --- | --- | --- | --- | --- |
| Grok Build | latest | Imagine + inline image display / library save (manual UI) | pass | 2026-08-18 |
| Other featured providers | — | not re-run for this build | unverified | 2026-08-18 |

This is an ad-hoc dogfooding package (ad-hoc signed; not Developer ID notarized).
Full provider matrix from 0.1.15 was not re-exercised.

### 中文摘要

**媒体素材库 + Grok Imagine**

- Grok Imagine 生成图可在对话中内联显示、点击预览，并在满意时手动「保存到素材库」；文件拷入应用自有目录，不随 Grok session 消失。
- 左轨第三项进入媒体素材库：瀑布流、排序、详情、右键显示/删除。标签与收藏本版不做。

**全局顶栏**

- 首页改为全宽顶栏容纳红绿灯与常用控件；列分割线从顶栏下方开始，不再切开红黄绿。

**发布说明**

本版为 ad-hoc 签名的 dogfooding 包（未走 Developer ID 公证）。除 Grok Imagine
相关 UI 冒烟外，其余 provider 矩阵未在本提交复测。

## [0.1.15]

### Provider removal: GitHub Copilot CLI

PengPilot no longer supports GitHub Copilot CLI. The provider is gone from
the model picker, Settings, provider detection, and the README; its icon
and driver branch are removed too. Sessions created by older builds with a
Copilot provider remain readable and open as Codex instead of disappearing.

### Provider verification (ad-hoc dogfooding release)

| Provider | CLI version | Test command | Result | Date |
| --- | --- | --- | --- | --- |
| Claude | 2.1.233 | `claude -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Codex | 0.147.0 | `codex exec --skip-git-repo-check "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Grok Build | latest | `grok -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Pi | latest | `pi "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| OMP | 17.3.5 | `omp -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Cursor | cursor-agent (latest) | `cursor-agent --trust -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| DeepSeek Harness | 0.1.0-rc.6 | `dsh --version` | pass (detection) | 2026-08-17 |
| OpenCode | 1.18.18 | `opencode run --auto "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Trae (CLI) | 0.120.52 | `traecli acp serve` handshake + `session/prompt` "Reply with exactly: PENG_OK" | pass | 2026-08-17 |
| Kiro (CLI) | 2.18.1 | `kiro-cli whoami` | installed; not authenticated | 2026-08-17 |

The dsh row covers CLI detection; a full Harness RPC turn was not exercised
for this build. Trae was driven through a real ACP session: initialize,
session/new (which returns the agent's model configuration), and a
`session/prompt` turn that completed with `end_turn` and returned the exact
expected text. Kiro is installed but not logged in, so its turn is pending
`kiro-cli login`. Amp, Antigravity, CodeBuddy, DevEco, Kimi, Qoder, Qwen,
Reasonix, Prime, and Hermes Agent are not installed on the build machine.
Tool, permission, resume, and cancel behavior was not re-exercised
end-to-end for this dogfooding build.

### 中文摘要

**移除 GitHub Copilot CLI 支持**

PengPilot 不再支持 GitHub Copilot CLI：模型选择器、设置、CLI 检测和
README 中不再出现该 provider，相关图标与驱动分支一并删除。旧版本创建
的 Copilot 会话仍可正常显示，并以 Codex 身份继续打开，不会从列表中
消失。

**Provider 验证（临时自用 dogfooding 版本）**

在打包提交上执行的真实 CLI 冒烟测试（检测 + 一轮流式回复）：Claude
2.1.233、Codex 0.147.0、Grok Build、Pi、OMP 17.3.5、Cursor
（cursor-agent）、OpenCode 1.18.18 全部通过；Trae 0.120.52 通过完整
ACP 回合（initialize → session/new 返回模型配置 → session/prompt
以 end_turn 完成并返回 PENG_OK）；DeepSeek Harness 0.1.0-rc.6 已验证
CLI 检测；Kiro 2.18.1 已安装但未登录，待 `kiro-cli login` 后补测。
Amp、Antigravity、CodeBuddy、DevEco、Kimi、Qoder、Qwen、Reasonix、
Prime、Hermes Agent 本机未安装，仍标注 unverified。工具调用、权限、
恢复与取消行为未在本版本端到端复测。

## [0.1.14]

### Stability fixes

- Quitting no longer flashes the whole screen black. PengPilot's window is
  transparent and blurred, and tearing it down used to leave a full-screen
  black flash on exit; the app now hides its window before teardown, and the
  final draft-save that runs during quit is bounded to two seconds so a
  stalled save can never hold the app on a dead screen.
- Starting the app no longer pops a browser window asking you to sign in to
  Kiro CLI. Model discovery for Kiro now probes `kiro-cli whoami` first: when
  the CLI is not authenticated, discovery degrades to the fallback model list
  instead of launching `kiro-cli chat`, which opens a browser login and
  blocks. The sign-in prompt still appears at the moment you actually start a
  Kiro session, where it belongs.

### Provider verification (ad-hoc dogfooding release)

| Provider | CLI version | Test command | Result | Date |
| --- | --- | --- | --- | --- |
| Claude | 2.1.233 | `claude -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Codex | 0.147.0 | `codex exec --skip-git-repo-check "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Grok Build | latest | `grok -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Pi | latest | `pi "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| OMP | 17.3.5 | `omp -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Cursor | cursor-agent (latest) | `cursor-agent --trust -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| DeepSeek Harness | 0.1.0-rc.6 | `dsh --version` | pass (detection) | 2026-08-17 |
| OpenCode | 1.18.18 | `opencode run --auto "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Trae (CLI) | 0.120.52 | `traecli acp serve` handshake + `session/prompt` "Reply with exactly: PENG_OK" | pass | 2026-08-17 |
| Kiro (CLI) | 2.18.1 | `kiro-cli whoami` | installed; not authenticated | 2026-08-17 |

The dsh row covers CLI detection; a full Harness RPC turn was not exercised
for this build. Trae was driven through a real ACP session: initialize,
session/new (which returns the agent's model configuration), and a
`session/prompt` turn that completed with `end_turn` and returned the exact
expected text. Kiro is installed but not logged in, so its turn is pending
`kiro-cli login`. Amp, Antigravity, CodeBuddy, DevEco, Kimi, Qoder, Qwen,
Reasonix, Prime, and Hermes Agent are not installed on the build machine.
Tool, permission, resume, and cancel behavior was not re-exercised
end-to-end for this dogfooding build.

### 中文摘要

**稳定性修复**

- 退出应用不再出现整屏黑闪。PengPilot 的窗口是透明+模糊的，退出销毁
  窗口表面时会在全屏闪过黑色；现在退出前先隐藏窗口再销毁，同时退出时
  的草稿保存限制在 2 秒内——保存卡住也不会让应用停在黑屏上。
- 启动应用不再自动弹出浏览器要求登录 Kiro CLI。Kiro 的模型发现现在
  先探测 `kiro-cli whoami`：CLI 未登录时直接回退到内置模型列表，不再
  执行 `kiro-cli chat`（它会打开浏览器登录页并阻塞等待）。登录提示只
  会在你真正开始 Kiro 会话时出现。

**Provider 验证（临时自用 dogfooding 版本）**

在打包提交上执行的真实 CLI 冒烟测试（检测 + 一轮流式回复）：Claude
2.1.233、Codex 0.147.0、Grok Build、Pi、OMP 17.3.5、Cursor
（cursor-agent）、OpenCode 1.18.18 全部通过；Trae 0.120.52 通过完整
ACP 回合（initialize → session/new 返回模型配置 → session/prompt
以 end_turn 完成并返回 PENG_OK）；DeepSeek Harness 0.1.0-rc.6 已验证
CLI 检测；Kiro 2.18.1 已安装但未登录，待 `kiro-cli login` 后补测。
Amp、Antigravity、CodeBuddy、DevEco、Kimi、Qoder、Qwen、Reasonix、
Prime、Hermes Agent 本机未安装，仍标注 unverified。工具调用、权限、
恢复与取消行为未在本版本端到端复测。

## [0.1.13]

### App icon refresh

- Replaced the app icon with the refreshed PengPilot mark: the macOS app
  bundle (`AppIcon.icns`, debug and release), the notification/about
  artwork, and the website + README icons (`app-icon`, apple-touch-icon,
  og-icon, favicon) all ship the new mark.

### 中文摘要

**应用图标更新**

- 应用图标替换为最新版 PengPilot 标志：macOS App 包（`AppIcon.icns`，
  调试与发布）、通知/关于对话框图标，以及网站与 README 图标（app-icon、
  apple-touch-icon、og-icon、favicon）全部换新。

## [0.1.12]

### Inbox fixes

- The Flagged tab no longer shows conversations that were never flagged: the
  sidebar's cached row snapshot now tracks the active collection and each
  session's workflow status and flag state, so switching tabs or toggling a
  flag moves rows immediately instead of showing a stale list.
- The session context menu is flag-aware: flagged conversations show
  「去掉旗标」and unflagged ones 「加旗标」; removing the flag drops the
  conversation out of the Flagged tab back into Open.
- Removing a conversation now asks for confirmation first: a card shows the
  session title, warns that the conversation and its checkpoints will be
  deleted permanently, and requires an explicit Remove click (or Enter) —
  Escape or Cancel dismisses it. No more accidental one-click deletions.

### Provider verification (ad-hoc dogfooding release)

Real CLI smoke tests on the packaged commit (detection + a streamed turn):

| Provider | CLI version | Test command | Result | Date |
| --- | --- | --- | --- | --- |
| Claude | 2.1.233 | `claude -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Codex | 0.147.0 | `codex exec --skip-git-repo-check "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Grok Build | latest | `grok -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Pi | latest | `pi "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| OMP | 17.3.5 | `omp -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| Cursor | cursor-agent (latest) | `cursor-agent --trust -p "Reply with exactly: PENG_OK"` | pass | 2026-08-17 |
| DeepSeek Harness | 0.1.0-rc.6 | `dsh web --host 127.0.0.1 --port 0` → ready line + HTTP 200 | pass (host) | 2026-08-17 |

OMP and Cursor were missed in the first verification pass (wrong command names
probed); DeepSeek Harness's profile is named `web`, not `default`. The dsh row
covers detection and host startup (the transport PengPilot drives); a full
Harness RPC turn was not exercised. OpenCode, Amp, and the JSON-CLI providers
remain unverified for this release (not installed on the build machine). Tool,
permission, resume, and cancel behavior was not re-exercised end-to-end for
this dogfooding build.

### 中文摘要

**收件箱（Inbox）修复**

- 「旗标」标签页不再显示从未加旗标的对话：侧栏行快照缓存现在追踪当前
  标签页、每个会话的流程状态与旗标状态，切换标签或切换旗标会立即移动
  对应行，不再展示过期列表。
- 会话右键菜单感知旗标状态：已旗标的对话显示「去掉旗标」，未旗标的显示
  「加旗标」；去掉旗标后对话从「旗标」标签页回到 Open 列表。
- 删除对话前增加二次确认：确认卡片显示会话标题，提示对话及其检查点将被
  永久删除，需明确点击「移除」（或按回车），Escape 或「取消」可关闭——
  不再有一次点击误删的风险。

**Provider 验证（临时自用 dogfooding 版本）**

在打包提交上执行的真实 CLI 冒烟测试（检测 + 一轮流式回复）：Claude
2.1.233、Codex 0.147.0、Grok Build、Pi、OMP 17.3.5、Cursor
（cursor-agent）全部通过；DeepSeek Harness 0.1.0-rc.6 验证到 host 启动与
HTTP 200（完整 RPC 回合未跑）。OpenCode、Amp 与 JSON-CLI 系列 provider 本
机未安装，仍标注 unverified。工具调用、权限、恢复与取消行为未在本版本
端到端复测。

## [0.1.11]

Syncs the shared codebase with the latest upstream Waku and ports its
streaming and interaction improvements.

### Performance

- Streamed responses now commit to layout in coalesced batches on a 120 ms
  frame floor instead of grapheme-budgeted chunks, and reasoning deltas ride
  the same cadence — a fast thinking stream no longer forces 40+ full
  re-renders a second.
- Newly appended streaming text fades in with a paint-only veil: the complete
  text enters layout immediately, so shaping, wrapping, and row heights never
  reflow while a response streams.
- Loaders share one pulse clock (~30 fps) with leases that park it when no
  loader is mounted, instead of each animation pinning the window at display
  rate.
- Sidebar, transcript, and right panel render as cached panes: a pulse or
  veil tick repaints only the island that owns it.
- A long live reasoning peek renders only its tail window, so a long think
  costs O(window) per frame instead of O(document).
- Code-block copy feedback and the live activity header now use cached
  fingerprints instead of re-deriving per frame.

### Interaction

- Agents can ask structured questions mid-turn (question cards with options,
  multi-select, and custom answers; Claude, Codex, OpenCode, ACP, and
  DeepSeek transports).
- Claude sessions expose a context-window choice (e.g. the 1M window opt-in)
  remembered per model.
- Codex gains a local `/fast` command that toggles the Fast service tier.
- Queued follow-ups can be steered individually from the composer.
- The selected provider's model catalog refreshes when its model picker opens.
- Composer commands show the command icon; wheel scrolling is contained
  inside nested scrollables; drag selection tracks at the window level so it
  extends past the input bounds.

### Polish and fixes

- Main-window position, size, and display are restored across launches.
- The embedded terminal uses the overlay scrollbar and measures cell width
  from the font.
- Fixes: shrink-wrapped user bubbles collapsing around quotes and lists,
  a char-boundary panic when sliding the live reasoning window, Markdown
  message width, unloaded history rendering the new-task prompt, and
  composer height overflow.
- The terminal and activity viewports cap their height and scroll with an
  overlay scrollbar.

### Provider verification (ad-hoc dogfooding release)

Real CLI smoke tests on the packaged commit (detection + a streamed turn):

| Provider | CLI version | Test command | Result | Date |
| --- | --- | --- | --- | --- |
| Claude | 2.1.233 | `claude -p "Reply with exactly: PENG_OK"` | pass | 2026-08-16 |
| Codex | 0.147.0 | `codex exec --skip-git-repo-check "Reply with exactly: PENG_OK"` | pass | 2026-08-16 |
| Grok Build | latest | `grok -p "Reply with exactly: PENG_OK"` | pass | 2026-08-16 |
| Pi | latest | `pi "Reply with exactly: PENG_OK"` | pass | 2026-08-16 |
| DeepSeek Harness | 0.1.0-rc.6 | installed, no profile configured | unverified | 2026-08-16 |

OMP, Cursor (cursor-agent), and DeepSeek Harness (profile `web`) were not
covered by that release's smoke pass; OpenCode, Amp, and the JSON-CLI
providers (Antigravity, CodeBuddy, Copilot, DevEco, Qwen, …) are not
installed on this machine. All of these remain unverified for 0.1.11. Tool,
permission, resume, and cancel behavior was not re-exercised end-to-end for
this dogfooding build.


## [unreleased]

## [0.1.10]

- Replace the new-task empty-state sparkle with a larger transparent PengPilot
  mark aligned with its heading.
- Use each provider site's browser favicon with one shared visual footprint in
  Settings and the Chat Box picker.
- Remove the unsupported OpenClaw integration.
- Remove `(CLI)` from Chat Box provider short names and label DeepSeek Harness
  without that suffix in Settings.
- Hide Prime Agent from the featured catalog and public provider lists until it
  passes real end-to-end validation and demonstrates user demand. Existing
  Prime Agent sessions remain compatible; any future restoration uses the same
  detected/undetected alphabetical ordering without default priority.
- Hide Kimi Code from the featured and public catalogs while keeping its
  runtime, `Kimi Code (CLI)` Settings label, `Kimi Code` picker label, and
  existing sessions compatible.
- Publish this Apple-silicon dogfooding build without treating missing real-CLI
  evidence as a blocker; unverified catalog entries remain experimental.

## [0.1.8]

- Focus the provider catalog on 15 CLIs, labeled with `(CLI)` in menus.
  Prime Agent is listed first as the recursive self-evolving agent; then
  Claude Code, Codex, Cursor, OpenCode, Grok Build, Kiro, Trae, DeepSeek
  Harness, Hermes Agent, OpenClaw, GitHub Copilot, Kimi Code, Pi, and Oh My
  Pi. Existing sessions on other CLIs still run.
- Replace the old Inbox navigation with All Tasks and Four-Quadrant boards,
  plus a compact Four-Quadrant shortcut beside New Task
- Add independently scrollable workflow lanes, persistent card drag-and-drop,
  and a global Completed or Archived lane shared by every quadrant
- Clarify workflow and quadrant menu labels and use one consistent
  Four-Quadrant icon across both entry points

## [0.1.7]

- Fix Computer Use startup failures caused by macOS Unix socket path limits
- Make each Board workflow lane independently scrollable
- Keep Board card titles to one truncated line and support persistent drag-and-drop
  across quadrants and workflow lanes

## [0.1.6]

- Add session Inbox views for unfinished, flagged, archived, and board work
- Expand the CLI catalog to 24 providers, including Prime Agent, with PATH-aware
  grouping and alphabetical sorting
- Remove anonymous product analytics, its setting, installation identifier,
  network sender, dependency, and release configuration
- Keep local Usage history while collecting no PengPilot telemetry
- Focus current releases on Apple silicon macOS; Intel Macs, Windows, and Linux
  remain untested and unsupported

## [0.1.5]

- Reduce the macOS ad-hoc release size while preserving runtime features
- Clarify PengPilot's independent identity and thank the Waku project

## [0.1.4]

- Replace the PengPilot app icon with the latest eagle-and-P artwork

## [0.1.3]

- Refine the PengPilot app icon with the latest eagle-and-P artwork

## [0.1.2]

- Replace the PengPilot app icon with the new red, blue, and silver artwork

## [0.1.1]

- Refresh the app icon with the new white-background artwork
- Match Hermes Agent icon sizing across provider settings and menus
- Correct PengPilot release URLs and local ad-hoc release configuration
- Refresh the bilingual README while preserving Waku attribution and GPL duties

## [0.1.0]

- Rebrand the app as PengPilot with new bundle IDs, data paths, and icon
- Add Oh My Pi, Kiro CLI, and Hermes Agent CLI providers
- Add Grok Build history to Usage charts and cost aggregation

## [0.0.13]

- Add DeepSeek Harness provider
- Render user message as Markdown and linkify bare URLs
- Share one resident OpenCode serve per workspace across sessions

## [0.0.12]

- Inherit the login-shell environment for provider commands
- Fix model traits across provider switches
- Keep branch change counts current and include untracked files
- Normalize SIGCHLD for provider children
- Fix Grok model discovery

## [0.0.11]

- Fix provider detection for CLIs installed through shell PATH managers such as
  nvm and fnm
- Show models registered by Pi extensions
- Fix the model picker closing when entering a space in search
- Fix duplicate transcript history and lost interaction mode when resuming ACP
  sessions

## [0.0.10]

- Fix crash in due to IME composition
- Fix typo

## [0.0.9]

- Add OpenCode Go support in usage popover
- Fix app icon
- Fix Cursor model detection

## [0.0.8]

- Initial release
