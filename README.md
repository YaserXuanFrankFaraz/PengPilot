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

### Agent catalog and verification

Install and authenticate at least one CLI before starting PengPilot. The app
detects CLIs on `PATH`; Settings → Providers also accepts an explicit binary
path. Detected providers appear first, followed by providers not found on
`PATH`.

The supported catalog is waku's official provider set minus Amp, plus
OMP, Hermes Agent, Trae CLI, and Kiro CLI — eleven providers in total. The
remaining provider stubs (Amp, Antigravity, CodeBuddy, DevEco, Kimi, Qoder,
Qoder CN, Qwen, QwenPaw, Reasonix, Prime) exist only so older sessions stay
readable; they are outside the supported catalog and are never verified.

Catalog presence means PengPilot can detect and route the CLI; it is not by
itself a compatibility guarantee. A provider becomes verified only after an
authenticated, real-CLI end-to-end test passes on that exact release commit.
PengPilot is currently a personal dogfooding build, so missing evidence does
not block an ad-hoc release; unverified entries remain experimental and are not
claimed as release-verified support.

| Agent | Command | v0.1.15 verification |
| --- | --- | --- |
| Claude Code (CLI) | `claude` | ✅ pass (2.1.233, 2026-08-17) |
| Codex (CLI) | `codex` | ✅ pass (0.147.0, 2026-08-17) |
| Cursor (CLI) | `cursor-agent` | ✅ pass (latest, 2026-08-17) |
| Grok Build (CLI) | `grok` | ✅ pass (latest, 2026-08-17) |
| Pi (CLI) | `pi` | ✅ pass (latest, 2026-08-17) |
| [Oh My Pi (CLI)](https://omp.sh/) | `omp` | ✅ pass (17.3.5, 2026-08-17) |
| DeepSeek Harness (CLI) | `dsh` | ✅ pass (host startup, 0.1.0-rc.6, 2026-08-17) |
| OpenCode (CLI) | `opencode` | ✅ pass (1.18.18, real turn, 2026-08-17) |
| [Kiro (CLI)](https://kiro.dev/cli/) | `kiro-cli` | Installed (2.18.1); not authenticated |
| Trae (CLI) | `traecli` | ✅ pass (0.120.52, real ACP turn, 2026-08-17) |
| [Hermes Agent (CLI)](https://github.com/NousResearch/hermes-agent) | `hermes` | Not installed; unverified |

Verification means a real authenticated CLI smoke test (detection + a
streamed turn) passed on the exact release commit. dsh is verified through
host startup (the transport PengPilot drives); a full Harness RPC turn was
not exercised. Tool, permission, resume, and cancel behavior is not covered
by the dogfooding smoke matrix.

Capabilities vary with each CLI and protocol. PengPilot exposes model
discovery, permissions, steering, rollback, forking, and Computer Use only
where the selected provider supports them. Sessions already started on other
CLIs continue to run; new work uses this catalog.

### Highlights

- Manage multiple local projects and independent, concurrently running agent
  tasks in one native app.
- Triage sessions through All Tasks and Four-Quadrant boards. Workflow lanes
  scroll independently, cards move across quadrants and progress states by
  drag-and-drop, and completed or archived work shares one global lane;
  placement and work-item metadata stay local.
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

Current releases target Apple silicon Macs running macOS 13 or newer. They are
personal dogfooding builds: Intel Macs, Windows, and Linux are not currently
tested or supported. Support for those platforms may be considered after the
Apple silicon macOS experience stabilizes.

### Privacy and local data

Projects, conversations, and settings are stored locally. PengPilot needs no
PengPilot account or hosted PengPilot service; each agent remains responsible
for its own authentication and network use.

PengPilot does not collect or send product analytics or telemetry. The Usage
page remains local: it reads supported CLI history and available provider quota
data on this Mac and does not report that information to PengPilot.

### Install

#### macOS

PengPilot currently requires an Apple silicon Mac running macOS 13 or newer.

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

PengPilot currently adds independent branding, bundle identifiers, local data
paths, All Tasks and Four-Quadrant workflows, a focused twelve-provider CLI
catalog, PATH-aware provider grouping, Grok Build usage history, and
PengPilot-specific release infrastructure. The exact changes are preserved in
this repository's Git history.

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

### 智能体目录与验证

启动 PengPilot 前，请至少安装并登录一个 CLI。PengPilot 会从 `PATH`
自动检测，也可以在「设置 → 服务商」中指定可执行文件路径。
已检测到的服务商显示在上方，未在 `PATH` 中检测到的显示在下方。

支持目录以 waku 官方服务商集合为基准，移除 Amp，并增加 OMP、Hermes
Agent、Trae CLI、Kiro CLI——共 11 家。其余保留的 provider 存根（Amp、
Antigravity、CodeBuddy、DevEco、Kimi、Qoder、Qoder CN、Qwen、QwenPaw、
Reasonix、Prime）仅用于让旧会话保持可读，不在支持目录内，也不做验证。

进入目录只表示 PengPilot 能检测并路由该 CLI，本身不等于兼容性承诺。只有在目标
发布提交上通过已登录真实 CLI 的端到端测试，才可称为该版本已验证。PengPilot
目前属于个人 dogfooding 版本，因此缺少证据不会阻止 ad-hoc 发布；未验证项目继续
标记为实验性支持，不宣称已经通过该版本验证。

| 智能体 | 命令 | v0.1.15 验证状态 |
| --- | --- | --- |
| Claude Code（CLI） | `claude` | ✅ 通过（2.1.233，2026-08-17） |
| Codex（CLI） | `codex` | ✅ 通过（0.147.0，2026-08-17） |
| Cursor（CLI） | `cursor-agent` | ✅ 通过（最新版，2026-08-17） |
| Grok Build（CLI） | `grok` | ✅ 通过（最新版，2026-08-17） |
| Pi（CLI） | `pi` | ✅ 通过（最新版，2026-08-17） |
| [Oh My Pi（CLI）](https://omp.sh/) | `omp` | ✅ 通过（17.3.5，2026-08-17） |
| DeepSeek Harness（CLI） | `dsh` | ✅ 通过（host 启动，0.1.0-rc.6，2026-08-17） |
| OpenCode（CLI） | `opencode` | ✅ 通过（1.18.18，真实回合，2026-08-17） |
| [Kiro（CLI）](https://kiro.dev/cli/) | `kiro-cli` | 已安装（2.18.1）；未登录 |
| Trae（CLI） | `traecli` | ✅ 通过（0.120.52，真实 ACP 回合，2026-08-17） |
| [Hermes Agent（CLI）](https://github.com/NousResearch/hermes-agent) | `hermes` | 未安装；未验证 |

验证指在打包的发布提交上通过真实鉴权 CLI 冒烟测试（检测 + 一轮流式回复）。
dsh 验证到 host 启动（即 PengPilot 驱动的传输层），未跑完整 Harness RPC
回合；工具调用、权限、恢复与取消行为不在本 dogfooding 冒烟矩阵覆盖范围内。

不同 CLI 和协议支持的能力并不完全相同。模型发现、权限交互、运行中追加指令、
回退、派生会话及 Computer Use 仅在对应服务商支持时显示。已经用其他 CLI
开始的会话会继续运行；新任务使用这份名单。

### 主要功能

- 在一个原生应用中管理多个本地项目和彼此独立、可并行运行的智能体任务。
- 通过「全任务看板」和「四象限看板」整理会话；各进度状态栏可独立滚动，
  会话卡片可拖拽到其他象限和进度状态，已完成或已归档任务统一进入全局状态栏；
  位置与工作项元数据均保存在本机。
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

当前版本仅面向运行 macOS 13 或更高版本的 Apple 芯片 Mac，定位为个人
dogfooding 版本。Intel Mac、Windows 和 Linux 目前均不测试、不承诺支持；待
Apple 芯片 macOS 体验稳定后，再考虑扩展到这些平台。

### 隐私与本地数据

项目、对话和设置默认保存在本机。PengPilot 不要求 PengPilot 账号或托管服务；
各智能体仍使用各自的身份认证和网络服务。

PengPilot 不采集、不发送产品分析或遥测数据。「用量」页面仍保留为本机功能：
它仅读取本机上受支持的 CLI 历史和服务商可用额度信息，不会向 PengPilot 上报。

### 安装

#### macOS

PengPilot 当前要求 Apple 芯片 Mac，并运行 macOS 13 或更高版本。

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

PengPilot 当前增加了独立品牌、Bundle ID、本地数据路径、全任务与四象限工作流、
12 个 CLI 服务商目录、基于 PATH 检测结果的服务商分组、Grok Build 用量历史，
以及 PengPilot 专属发布基础设施。所有具体修改均保留在本仓库的 Git 历史中。

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
