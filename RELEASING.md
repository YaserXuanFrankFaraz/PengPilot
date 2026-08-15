# Releasing PengPilot

PengPilot auto-updates with [Sparkle](https://sparkle-project.org). Releases can live in
a **Cloudflare R2** bucket or GitHub Releases. New users
download a notarized **`.dmg`**; existing users get smaller in-app updates
(binary deltas when available) via Sparkle, which reads the appcast at
the configured `appcast.xml`, verifies each build's EdDSA signature,
and installs it. One release command produces and publishes both.

Once set up, cutting a release is:

```sh
bun run release
```

- Updater code: [`src/updater.rs`](src/updater.rs) — loads the embedded
  Sparkle.framework at runtime and starts `SPUUpdater` with PengPilot's custom user
  driver. Available updates appear in the sidebar footer; download, signature
  verification, install, and relaunch remain owned by Sparkle. **Check for
  Updates…** lives in the app menu, and the **Automatic updates** toggle in
  Settings → General mirrors Sparkle's persisted setting.
- Feed URL + public key: [`resources/Info.plist`](resources/Info.plist)
  (`SUFeedURL`, `SUPublicEDKey`).
- Framework embedding + pinned Sparkle version:
  [`scripts/bundle.sh`](scripts/bundle.sh) (bump `sparkle_version` and
  `sparkle_sha256` together; the distribution is cached under
  `.pengpilot-cache/sparkle/`).
- Release automation: [`scripts/release.ts`](scripts/release.ts),
  [`scripts/appcast.ts`](scripts/appcast.ts),
  [`scripts/changelog.ts`](scripts/changelog.ts).
- GitHub Actions: [`.github/workflows/release.yml`](.github/workflows/release.yml)
  builds Linux (x86_64, arm64) and macOS archives on a `v*` tag and opens a
  draft GitHub release;
  [`.github/workflows/sync-release.yml`](.github/workflows/sync-release.yml)
  copies published assets into the R2 bucket.

---

## One-time setup

The release runs on [Bun](https://bun.sh) and needs
[`create-dmg`](https://github.com/create-dmg/create-dmg) and
[rclone](https://rclone.org) (`brew install bun create-dmg rclone`).

### 1. Sparkle signing keys

Updates are signed with an ed25519 key; the private half stays in the login
keychain and the public half ships in Info.plist as `SUPublicEDKey`.

The current checkout intentionally has no `SUPublicEDKey`. Configure a
PengPilot-specific key before publishing the first automatic update. Never
reuse Waku's or another application's private signing key.

Generate or restore the PengPilot key with the Sparkle tools (they land in
`.pengpilot-cache/sparkle/<version>/bin` after any build, or download the release from
[sparkle-project/Sparkle](https://github.com/sparkle-project/Sparkle/releases)):

```sh
./bin/generate_keys --account pengpilot
./bin/generate_keys -p --account pengpilot
```

Add the printed public key to `resources/Info.plist` as `SUPublicEDKey`. Keep
the private key in the login keychain and in a secure backup.

> ⚠️ Lose the private key and existing installs can never update again. Keep
> the backup current.

If the account name changes, pass the same account to `generate_appcast` in
`scripts/appcast.ts`. Existing installs trust only the public key embedded in
their build, so key rotation requires an intentionally staged migration.

### 2. Developer ID signing + notarization

Copy `.env.example` to `.env` and replace the signing placeholder. The script
notarizes with the `NOTARY` keychain profile by default. On a fresh machine:

```sh
cp .env.example .env
xcrun notarytool store-credentials NOTARY \
  --apple-id you@example.com --team-id YOUR_APPLE_TEAM_ID
```

Override the environment with `--signing-identity`, or change the notary
profile with `--notary-profile` / `PENGPILOT_NOTARY_PROFILE`.

### 3. Cloudflare R2 bucket + domain  ← **still to do once**

1. Create the bucket **`pengpilot-releases`** (Cloudflare dashboard → R2 → Create
   bucket). The release script will not create it — a bucket-scoped API token
   can't.
2. Attach a PengPilot release domain to the bucket (bucket →
   Settings → Custom Domains). This serves objects publicly at
   `https://your-release-domain/<file>`.
3. Make sure the R2 API token behind the `r2` rclone remote covers this bucket
   (R2 → Manage API Tokens → Object Read & Write). The remote already exists
   for PengPilot; if `rclone lsf r2:pengpilot-releases --s3-no-check-bucket` returns
   *AccessDenied* after the bucket exists, extend the token's bucket list.

The rclone remote itself (`~/.config/rclone/rclone.conf`, type S3, provider
Cloudflare, `no_check_bucket = true`) is shared with kero and needs no change.

---

## Release gates — blocking

These gates apply to every public build, including ad-hoc dogfooding releases.
Do not create a tag, upload artifacts, or publish a release while any gate is
open.

### 1. Record real CLI evidence

`ProviderKind::FEATURED` is the visible dogfooding catalog, not proof that every
entry passed a real CLI test. On the exact commit being packaged, record any
authenticated real integration tests that were actually run. A normal
`cargo test` run that reports an `#[ignore]`d provider test is not real-CLI
evidence.

Each provider must demonstrate:

1. detection from the same environment used by the packaged app;
2. model or agent discovery;
3. session start and streamed response completion;
4. tool activity and permission behavior when the CLI supports them;
5. a second turn or native resume;
6. stop/cancel and visible error recovery; and
7. every provider-specific capability advertised by PengPilot.

Record one row per provider in the release notes before packaging:

| Provider | CLI version | Test command | Result | Date |
| --- | --- | --- | --- | --- |
| Every `ProviderKind::FEATURED` entry | exact version | explicit real test | pass | YYYY-MM-DD |

Missing credentials, unavailable services, or absent dedicated tests do not
block an ad-hoc personal dogfooding release. They must remain disclosed as
unverified, and the release must not claim verified support. A failed real test
does block that provider claim: fix it or hide the provider from new work while
retaining old-session compatibility.

### 2. Clean and audit the package footprint

Perform this audit before implementing each new feature and again immediately
before its release build. Delete obsolete functionality and bundled material
first. New dependencies, frameworks, helpers, runtimes, models, duplicate
artwork, fallback catalogs, and assets need a current shipped use; speculative
or development-only files do not belong in the app.

Before packaging, remove only stale assembled outputs for the target version:

```sh
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
rm -f "dist/PengPilot-${version}.dmg" "dist/PengPilot-${version}.zip"
```

`scripts/bundle.sh` recreates the `.app` directory, while `release.ts` recreates
the DMG and update staging directory. Do not use `cargo clean` as package
hygiene: it deletes compiler caches but does not remove shipped bytes. Do not
use `--skip-build` for a public release unless both release executables were
freshly built from the exact commit and no source or bundled resource is newer.

After packaging, inspect the actual App and mounted DMG, not only build output:

```sh
du -sk target/release/PengPilot.app
find target/release/PengPilot.app -type f -exec du -h {} + | sort -h | tail -30
stat -f '%z %N' dist/PengPilot-*.dmg dist/PengPilot-*.zip
```

Record App, DMG, and ZIP sizes beside the previous release. Remove unused
architectures, debug symbols, development headers/modules, caches, logs,
temporary data, stale resources, empty embedded products, and frameworks not
used by that release mode. Any unexplained size increase blocks release;
necessary growth must be identified in the release notes.

### 3. Final artifact verification

The full test suite, provider matrix, formatting checks, mounted-DMG contents,
bundle metadata, Apple-silicon architecture, signatures, notarization state,
and SHA-256 checksums must all match the release notes. Generated `target/` and
`dist/` outputs stay out of Git. Current product support is Apple-silicon
macOS only; retained Intel, Windows, or Linux code and CI do not imply support
and their artifacts must not be attached to a public PengPilot release.

---

## Cutting a release

1. **Close every release gate above.** Package-size comparison is mandatory;
   provider evidence must be honest but may remain incomplete for an ad-hoc
   personal dogfooding release.
2. **Bump `version` in `Cargo.toml`** — the single source of truth.
   `CFBundleShortVersionString` is the version, and `CFBundleVersion` is
   derived from it (`major*1e6 + minor*1e3 + patch`, so `0.2.0` → `2000`),
   which keeps Sparkle's build-number comparison monotonic without a manual
   counter. Prerelease versions (`-beta.1`) are refused for publishing — the
   appcast serves one stable channel.
3. **Write the release notes** — add a `## [<version>]` section at the top of
   [`CHANGELOG.md`](CHANGELOG.md).
4. **Run it:**
   ```sh
   bun run release
   ```

The script checks R2 up front (bucket reachable, version not already
published), builds and signs the app via `scripts/bundle.sh release`, verifies
the bundled JS REPL and computer-use helper, builds the styled DMG, notarizes
and staples DMG + app, zips the app for Sparkle, pulls the recent archives
from R2 so `generate_appcast` can build binary deltas, attaches the changelog
section as release notes, regenerates the signed `appcast.xml`, and uploads
everything with immutable cache headers (the appcast itself stays
`max-age=300`). When it finishes:

- **Download link**: `https://github.com/YaserXuanFrankFaraz/PengPilot/releases/latest/download/PengPilot-<version>.dmg`
- **In-app updates**: served from the same origin via the appcast.

Test by keeping an older build around, launching it, and choosing
**Check for Updates…**.

### GitHub draft release + R2 sync

The Release workflow is manual until PengPilot has Developer ID, notarization,
and Sparkle signing credentials. Its macOS job runs `bun run release --local`
and writes the same artifacts as a local release:

- `PengPilot-<version>.dmg`
- `PengPilot-<version>.zip`
- `appcast.xml` (Sparkle-signed)

The workflow opens (or updates) a **draft** GitHub release with those files and
the matching `CHANGELOG.md` section. Intel macOS, Windows, and Linux source may
remain in the repository, but current releases do not build, test, or publish
their artifacts.

After the paid release infrastructure is configured, manually running
**Sync release** from Actions uploads the selected release assets to the
`pengpilot-releases` R2 bucket. Configure these repository secrets first:

| Secret | Purpose |
| --- | --- |
| `PENGPILOT_SIGNING_IDENTITY` | Developer ID identity selector |
| `APPLE_CERTIFICATE` | base64-encoded Developer ID Application `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | password for that `.p12` |
| `APPLE_ID` | Apple ID used by `notarytool` |
| `APPLE_APP_SPECIFIC_PASSWORD` | app-specific password for that Apple ID |
| `APPLE_TEAM_ID` | Developer Team ID |
| `SPARKLE_PRIVATE_KEY` | EdDSA private key for `generate_appcast` |
| `R2_ACCOUNT_ID` | Cloudflare account id for the R2 API |
| `R2_ACCESS_KEY_ID` | R2 Object Read & Write token |
| `R2_SECRET_ACCESS_KEY` | matching secret |
| `R2_BUCKET` | optional; defaults to `pengpilot-releases` |

### Options

| Flag / Env | Default | Purpose |
| --- | --- | --- |
| `--local` | — | build, notarize, and write the DMG + zip without publishing |
| `--force` | — | re-publish a version that already exists in R2 |
| `--adhoc`, `--skip-notarize` | — | local test builds (imply `--local`) |
| `--skip-build` | — | reuse existing release binaries |
| `--build-number <n>` / `PENGPILOT_BUILD_NUMBER` | derived | `CFBundleVersion` override |
| `PENGPILOT_R2_REMOTE` | `r2` | rclone remote name |
| `PENGPILOT_R2_BUCKET` | `pengpilot-releases` | R2 bucket |
| `PENGPILOT_DOWNLOAD_URL_PREFIX` | GitHub latest release assets | base URL in the appcast |
| `PENGPILOT_HISTORY_COUNT` | `15` | recent archives pulled for delta generation |
| `PENGPILOT_NO_HISTORY=1` | — | skip pulling old archives (full updates only) |
| `SPARKLE_BIN` | the `.pengpilot-cache` copy | Sparkle tools directory |

---

## Notes

- **Two artifacts per release:** the notarized `.dmg` (what people download)
  and a `.zip` (what Sparkle installs, plus `.delta` files against recent
  builds). Only the zip family appears in the appcast; point download buttons
  at the DMG.
- **Debug builds never update themselves.** `Updater::init` returns `None`
  under `debug_assertions`, so the dev watcher's app can't offer to replace
  itself with a production PengPilot. Set `PENGPILOT_FORCE_UPDATER=1` to exercise the
  real Sparkle flow from a debug bundle anyway. A bare `cargo run` binary has
  no embedded framework and also degrades to no updater. For UI-only testing,
  start the watcher with `PENGPILOT_PREVIEW_UPDATE=1`; the sidebar immediately
  shows an available update and clicking it changes to the spinner without
  installing anything. The preview flag fakes only that sidebar result;
  **Check for Updates…** still uses the embedded Sparkle framework and its
  real standard window.
- **Automatic and explicit checks have separate presentation.** Scheduled
  checks stay silent until the sidebar update button appears. Choosing
  **Check for Updates…** promotes an existing silent result into Sparkle's
  standard updater window, or shows its checking progress while an automatic
  check finishes. With no automatic session active, it starts Sparkle's
  standard user-initiated check directly.
- **First-run consent:** Sparkle shows its one-time "check automatically?"
  prompt on the second launch. The Settings → General toggle reads and writes
  the same persisted value.
- **PengPilot isn't sandboxed**, so Sparkle's XPC services are unnecessary;
  `bundle.sh` strips them (plus headers/modules) from the embedded framework
  and re-signs the rest with the app's identity — hardened-runtime library
  validation requires the identities to match.
- **Old archives stay in R2** so far-behind users can still be served; only
  the recent history is staged locally under `dist/updates/` (git-ignored).
- **Platform artifacts:** keep the bucket layout flat and platform-tagged by
  artifact name/extension — today's macOS names
  (`PengPilot-<v>.dmg`, `PengPilot-<v>.zip`, `appcast.xml`) must keep their URLs.
  Intel macOS, Windows, and Linux code stays dormant: do not build or publish
  those artifacts until their support phase is explicitly reopened.
