# PengPilot

<p align="center">
  <img src="website/public/app-icon.png" width="128" alt="PengPilot app icon">
</p>

<p align="center">
  A fast native desktop control plane for local coding agents.<br>
  面向本地编程智能体的高性能原生桌面控制台。
</p>

<p align="center">
  <a href="#english">English</a> · <a href="#简体中文">简体中文</a>
</p>

## English

PengPilot is a Rust and
[GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) desktop app
for running local coding-agent CLIs. It keeps projects, tasks, transcripts, and
settings on your machine while preserving each provider's native session and
structured event stream.

> [!IMPORTANT]
> PengPilot is an independent project with its own name, brand, product
> direction, design, releases, and community. It is not affiliated with or
> endorsed by Waku. PengPilot began as a GPL-3.0 modification of
> [Waku](https://github.com/egoist/waku) on August 14, 2026. This repository
> retains Waku's Git history, required attribution, and PengPilot-specific
> changes.

[Download PengPilot](https://github.com/YaserXuanFrankFaraz/PengPilot/releases)

### Supported agents

Install and authenticate at least one CLI before starting PengPilot. The app
detects CLIs on `PATH`; Settings → Providers also accepts an explicit binary
path.

| Agent | Command |
| --- | --- |
| [Amp](https://ampcode.com/) | `amp` |
| Claude Code | `claude` |
| Codex CLI | `codex` |
| Cursor CLI | `cursor-agent` |
| DeepSeek Harness | `dsh` |
| OpenCode | `opencode` |
| Grok Build | `grok` |
| Pi | `pi` |
| [Oh My Pi](https://omp.sh/) | `omp` |
| [Kiro CLI](https://kiro.dev/cli/) | `kiro-cli` |
| [Hermes Agent](https://github.com/NousResearch/hermes-agent) | `hermes` |

Capabilities vary with each CLI and protocol. PengPilot exposes model
discovery, permissions, steering, rollback, forking, and Computer Use only
where the selected provider supports them.

### Highlights

- Manage multiple local projects and independent, concurrently running agent
  tasks in one native app.
- Keep long-lived provider sessions across turns, with streamed responses,
  reasoning, tool activity, approvals, and provider-native continuity.
- Select models, reasoning effort, service tier, build/plan interaction, and
  supervised or autonomous access modes when supported.
- Steer a running turn or queue the next message without losing the task's
  context.
- Use Git-aware checkpoints, conversation rewind/fork, branch switching,
  worktrees, change review, commit, and push workflows.
- Work beside the conversation with Browser, Terminal, Files, and Review
  surfaces; edit files and inspect uncommitted, committed, or branch diffs.
- Discover, inspect, enable, disable, and invoke reusable `SKILL.md` skills
  across supported agent ecosystems.
- Monitor background commands and subagents, including status, output, and
  stop controls.
- Review local usage history, token and cost breakdowns, and available account
  quota data, including Grok Build history.
- On macOS, optionally enable isolated Computer Use for Codex, OpenCode, Grok,
  and Pi, with per-app approval plus Screen Recording and Accessibility grants.
- Support Sparkle updates in properly signed macOS release builds. Current
  ad-hoc GitHub releases use manual updates.

The embedded browser and Computer Use are currently macOS-only. Core agent
sessions, projects, transcripts, skills, usage, diffs, file editing, and the
terminal run natively on Linux.

### Privacy and local data

Projects, conversations, and settings are stored locally. PengPilot needs no
PengPilot account or hosted PengPilot service; each agent remains responsible
for its own authentication and network use.

Production builds may include optional anonymous product analytics. The
setting can be disabled under Settings → General. Prompts, responses, project
names, file paths, and other personal content are excluded from those events.

### Install

#### macOS

PengPilot requires macOS 13 or newer.

1. Download the latest `.dmg` from
   [GitHub Releases](https://github.com/YaserXuanFrankFaraz/PengPilot/releases).
2. Open the DMG and drag `PengPilot.app` to `Applications`.
3. Install and authenticate the agent CLI or CLIs you want to use.
4. Launch PengPilot and confirm detection under Settings → Providers.

Current GitHub builds are ad-hoc signed and not notarized. On first launch,
Control-click PengPilot and choose Open. If macOS still blocks it, use System
Settings → Privacy & Security → Open Anyway. Updates are installed manually
from GitHub Releases.

Computer Use requires separate macOS Screen Recording and Accessibility
permission for PengPilot's isolated helper.

#### Linux

Linux supports Wayland and X11 and requires a working Vulkan driver. Use a
release archive when available, or build from source after installing the
native packages listed in [CONTRIBUTING.md](CONTRIBUTING.md).

### Development

Development requires
[Rust 1.96 or newer](https://www.rust-lang.org/tools/install) and
[Bun](https://bun.sh/).

```sh
bun install
bun run dev
```

The watcher builds, launches, and relaunches the debug app after source
changes. Do not run a second watcher against the same checkout.

See [CONTRIBUTING.md](CONTRIBUTING.md) for platform prerequisites and checks,
and [RELEASING.md](RELEASING.md) for signed release packaging.

### PengPilot direction

PengPilot has its own product direction, visual identity, bundle identifiers,
local data paths, provider integrations, and release infrastructure. Current
work includes Oh My Pi, Kiro CLI, and Hermes Agent integrations plus Grok Build
usage history. Exact changes are preserved in this repository's Git history.

### Thanks and attribution

PengPilot gratefully thanks egoist and every
[Waku](https://github.com/egoist/waku) contributor for the foundation on which
this project began. PengPilot is independently named, developed, maintained,
and released; it is not a Waku product or official successor. Copyright
remains with the respective Waku and PengPilot contributors. Upstream
copyright, license, attribution, and no-warranty notices remain in this
repository and its history.

### License and redistribution obligations

Waku is licensed under the
[GNU General Public License version 3 only](https://github.com/egoist/waku/blob/main/LICENSE).
As a modified version, PengPilot is licensed as a whole under the same
**GPL-3.0-only** terms. The complete, controlling license text is included in
[LICENSE](LICENSE). This summary does not replace the license.

If you copy, modify, or distribute PengPilot:

1. Keep the GPL license, copyright notices, attribution notices, and
   no-warranty notices intact.
2. Prominently state that your version is modified and give the relevant
   modification date.
3. License the entire covered work under GPL-3.0-only without imposing further
   restrictions on recipients' GPL rights.
4. Give recipients a copy of GPLv3.
5. When conveying binaries or other object code, provide the complete,
   machine-readable Corresponding Source required by GPLv3 section 6,
   including the source and scripts needed to build, install, run, and modify
   that exact version. Network distribution must offer equivalent source
   access at no additional charge and keep clear source directions beside the
   binaries for as long as GPLv3 requires.
6. Provide Installation Information when GPLv3 section 6 requires it for a
   User Product, and preserve required third-party notices.
7. Preserve Appropriate Legal Notices in interactive interfaces when GPLv3
   section 5(d) requires them.

A public source repository alone does not cure a binary distribution whose
source is missing or does not correspond to the distributed build. Each
distributor is responsible for satisfying GPLv3 for the copies it conveys.

PengPilot is provided **without warranty**, to the extent permitted by law.
See GPLv3 sections 15 and 16 in [LICENSE](LICENSE).

---

## 简体中文

PengPilot 是一款使用 Rust 和
[GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) 构建的桌面应用，
用于统一运行本机上的编程智能体 CLI。项目、任务、对话记录和设置保存在本机，
同时保留各服务商原生会话及结构化事件流。

> [!IMPORTANT]
> PengPilot 是拥有独立名称、品牌、产品方向、设计、发行与社区的独立项目，
> 与 Waku 无隶属关系，也未获得其官方背书。PengPilot 于 2026 年 8 月 14 日
> 从 [Waku](https://github.com/egoist/waku) 的 GPL-3.0 修改版本起步。
> 本仓库保留 Waku 的完整 Git 历史、必要署名及 PengPilot 专属修改。

[下载 PengPilot](https://github.com/YaserXuanFrankFaraz/PengPilot/releases)

### 支持的智能体

启动 PengPilot 前，请至少安装并登录一个 CLI。PengPilot 会从 `PATH`
自动检测，也可以在「设置 → 服务商」中指定可执行文件路径。

| 智能体 | 命令 |
| --- | --- |
| [Amp](https://ampcode.com/) | `amp` |
| Claude Code | `claude` |
| Codex CLI | `codex` |
| Cursor CLI | `cursor-agent` |
| DeepSeek Harness | `dsh` |
| OpenCode | `opencode` |
| Grok Build | `grok` |
| Pi | `pi` |
| [Oh My Pi](https://omp.sh/) | `omp` |
| [Kiro CLI](https://kiro.dev/cli/) | `kiro-cli` |
| [Hermes Agent](https://github.com/NousResearch/hermes-agent) | `hermes` |

不同 CLI 和协议支持的能力并不完全相同。模型发现、权限交互、运行中追加指令、
回退、派生会话及 Computer Use 仅在对应服务商支持时显示。

### 主要功能

- 在一个原生应用中管理多个本地项目和彼此独立、可并行运行的智能体任务。
- 跨轮次保持长生命周期服务商会话，流式呈现回答、推理、工具活动和权限请求。
- 在服务商支持时选择模型、推理强度、服务等级、构建/规划交互方式，以及监督或
  自动化访问模式。
- 智能体工作时可追加指令，或将下一条消息排队，不丢失当前任务上下文。
- 提供 Git 检查点、对话回退/派生、分支切换、工作树、变更审查、提交和推送流程。
- 对话旁可使用浏览器、终端、文件和审查面板；支持编辑文件，并比较未提交、
  已提交或分支差异。
- 跨支持的智能体生态发现、检查、启用、停用和调用可复用的 `SKILL.md` Skills。
- 查看后台命令及子智能体的状态与输出，并可停止仍在运行的任务。
- 查看本地用量历史、Token/费用拆分及可用的账户额度信息，包括 Grok Build 历史。
- macOS 上可为 Codex、OpenCode、Grok 和 Pi 启用隔离的 Computer Use，并通过
  应用白名单、屏幕录制和辅助功能权限控制访问。
- 使用正式签名的 macOS Release 构建支持 Sparkle 更新；当前 GitHub ad-hoc
  发行版采用手动更新。

内嵌浏览器与 Computer Use 目前仅支持 macOS。Linux 原生支持智能体会话、项目、
对话记录、Skills、用量、差异审查、文件编辑和终端。

### 隐私与本地数据

项目、对话和设置默认保存在本机。PengPilot 不要求 PengPilot 账号或托管服务；
各智能体仍使用各自的身份认证和网络服务。

正式版本可能包含可选的匿名产品统计，可在「设置 → 通用」中关闭。统计事件不包含
提示词、回答、项目名称、文件路径或其他个人内容。

### 安装

#### macOS

PengPilot 要求 macOS 13 或更高版本。

1. 从 [GitHub Releases](https://github.com/YaserXuanFrankFaraz/PengPilot/releases)
   下载最新 `.dmg`。
2. 打开 DMG，将 `PengPilot.app` 拖入「应用程序」。
3. 安装并登录需要使用的智能体 CLI。
4. 启动 PengPilot，在「设置 → 服务商」中确认检测结果。

当前 GitHub 构建使用 ad-hoc 签名且未经 Apple 公证。首次启动时，请按住 Control
点击 PengPilot 并选择「打开」。如果 macOS 仍然拦截，请前往「系统设置 →
隐私与安全性 → 仍要打开」。后续版本需从 GitHub Releases 手动下载安装。

Computer Use 需要为 PengPilot 的隔离辅助程序单独授予 macOS「屏幕录制」和
「辅助功能」权限。

#### Linux

Linux 支持 Wayland 和 X11，并需要可用的 Vulkan 驱动。可以使用已发布的归档包，
或先安装 [CONTRIBUTING.md](CONTRIBUTING.md) 列出的系统依赖，再从源码构建。

### 开发

开发需要 [Rust 1.96 或更高版本](https://www.rust-lang.org/tools/install) 和
[Bun](https://bun.sh/)。

```sh
bun install
bun run dev
```

监听器会在源码变化后构建、启动并重新启动 Debug 应用。不要在同一工作区运行
第二个监听器。

平台依赖和检查命令见 [CONTRIBUTING.md](CONTRIBUTING.md)，签名发行流程见
[RELEASING.md](RELEASING.md)。

### PengPilot 的独立方向

PengPilot 拥有自己的产品方向、视觉品牌、Bundle ID、本地数据路径、服务商集成与
发布基础设施。目前的独立工作包括 Oh My Pi、Kiro CLI、Hermes Agent 集成及
Grok Build 用量历史。所有具体修改均保留在本仓库的 Git 历史中。

### 致谢与署名

PengPilot 衷心感谢 egoist 与所有 [Waku](https://github.com/egoist/waku)
贡献者，为本项目的起步奠定基础。PengPilot 拥有独立名称，由自身团队独立开发、
维护和发行；它不是 Waku 产品，也不是其官方继任者。版权分别归 Waku 与 PengPilot
的相应贡献者所有。本仓库及其历史继续保留上游版权、许可证、署名与无担保声明。

### 许可证与再分发义务

Waku 使用
[GNU 通用公共许可证第 3 版（仅此版本）](https://github.com/egoist/waku/blob/main/LICENSE)。
作为修改版本，PengPilot 整体继续使用相同的 **GPL-3.0-only** 条款。完整且具有
约束力的许可证文本位于 [LICENSE](LICENSE)，本节仅为摘要，不能替代许可证原文。

复制、修改或分发 PengPilot 时：

1. 保留 GPL 许可证、版权声明、署名声明及无担保声明。
2. 明确说明版本已经修改，并给出相关修改日期。
3. 将整个受保护作品继续以 GPL-3.0-only 授权，不得对接收者的 GPL 权利增加限制。
4. 向接收者提供一份 GPLv3 许可证。
5. 分发二进制或其他目标代码时，按照 GPLv3 第 6 条提供该精确版本完整、机器可读的
   对应源代码，包括构建、安装、运行和修改所需的源代码与脚本。通过网络分发时，
   必须以不额外收费的方式提供等价源码访问，并在 GPL 要求的期限内，于二进制附近
   保留清晰的源码获取说明。
6. GPLv3 第 6 条对 User Product 有要求时，提供 Installation Information，并保留
   第三方组件要求的声明。
7. GPLv3 第 5(d) 条适用时，保留交互界面中的 Appropriate Legal Notices。

仅公开源码仓库，并不能补救“源码缺失”或“源码与已分发二进制不对应”的问题。
每个分发者都应对其提供的副本履行 GPLv3 义务。

在法律允许的最大范围内，PengPilot **不提供任何担保**。完整条款见
[LICENSE](LICENSE) 第 15、16 条。
