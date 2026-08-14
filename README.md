# PengPilot

PengPilot is a fast, native desktop app for working with local coding agents.
It is built in Rust with
[GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) and keeps
projects, sessions, and transcripts on your machine.

> [!IMPORTANT]
> PengPilot is an independent modified version of
> [Waku](https://github.com/egoist/waku), not an official Waku release. The
> PengPilot modifications began on August 14, 2026. This repository retains
> Waku's Git history and identifies the changes made by PengPilot contributors.

[PengPilot releases](https://github.com/YaserXuanFrankFaraz/PengPilot/releases)

## Supported agents

PengPilot works with:

- [Amp](https://ampcode.com/)
- Claude Code
- Codex CLI
- Cursor CLI
- DeepSeek Harness
- Grok Build
- OpenCode
- Pi
- [Oh My Pi](https://omp.sh/)
- [Kiro CLI](https://kiro.dev/cli/)
- [Hermes Agent](https://github.com/NousResearch/hermes-agent)

Install and authenticate at least one supported agent CLI before starting
PengPilot. PengPilot detects available CLIs automatically and uses each
provider's native structured protocol and session continuity.

## Highlights

- Keep projects and independent agent sessions in one native app.
- Switch models, reasoning effort, and access modes from a shared interface.
- Queue or steer follow-up messages while an agent is working.
- Rewind Git-backed tasks with conversation-aware checkpoints.
- Review local usage history, including Grok Build usage.
- Store app state locally, with no PengPilot account or remote service required.

Compared with its Waku base, PengPilot adds Oh My Pi, Kiro CLI, and Hermes
Agent integrations, Grok Build usage history, and independent PengPilot
branding and application data.

## Development

Development is supported on macOS and Linux and requires
[Rust 1.96 or newer](https://www.rust-lang.org/tools/install) and
[Bun](https://bun.sh/). Linux supports both Wayland and X11; install the native
build prerequisites listed in [CONTRIBUTING.md](CONTRIBUTING.md) first.

```sh
bun install
bun run dev
```

The embedded browser and experimental computer-use integration currently
remain macOS-only. Agent sessions, projects, transcripts, skills, usage,
diffs, file editing, and the terminal run natively on Linux.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the development workflow and checks.
Release maintainers should also read [RELEASING.md](RELEASING.md).

## Upstream and attribution

PengPilot is based on [Waku](https://github.com/egoist/waku) by egoist and Waku
contributors. Copyright remains with the respective Waku and PengPilot
contributors. Upstream copyright, license, attribution, and no-warranty
notices are retained in this repository and its history.

The `upstream` Git remote should point to `https://github.com/egoist/waku.git`
when synchronizing future Waku changes.

## License and redistribution obligations

Waku is licensed under the
[GNU General Public License version 3 only](https://github.com/egoist/waku/blob/main/LICENSE).
As a modified version, PengPilot is licensed as a whole under the same
**GPL-3.0-only** terms. The complete, controlling license text is included in
[LICENSE](LICENSE). This README summarizes practical obligations; the license
text controls if there is any conflict.

If you copy, modify, or distribute PengPilot:

1. Keep the GPL license, copyright notices, attribution notices, and
   no-warranty notices intact.
2. Prominently state that your version is modified and provide the relevant
   modification date.
3. License the entire covered work under GPL-3.0-only and do not impose further
   restrictions on recipients' GPL rights.
4. Give recipients a copy of the GPLv3 license.
5. When distributing binaries or other object code, provide the complete,
   machine-readable Corresponding Source required by GPLv3 section 6,
   including the source and scripts needed to build, install, run, and modify
   that exact version. If binaries are offered from a network location, give
   equivalent source access at no additional charge and keep clear source
   directions next to the binaries for as long as the GPL requires.
6. Provide Installation Information when GPLv3 section 6 requires it for a
   User Product, and preserve any notices required by third-party components.
7. Preserve Appropriate Legal Notices in interactive interfaces when GPLv3
   section 5(d) requires them.

Publishing a source repository does not by itself cure a binary distribution
whose source is missing or does not correspond to the distributed build. Each
distributor is responsible for satisfying GPLv3 for the copies they convey.

PengPilot is provided **without warranty**, to the extent permitted by law.
See GPLv3 sections 15 and 16 in [LICENSE](LICENSE) for the full warranty and
liability terms.
