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
