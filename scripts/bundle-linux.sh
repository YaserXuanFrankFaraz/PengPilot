#!/usr/bin/env bash

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

target_dir="${CARGO_TARGET_DIR:-target}"
version="$(cargo metadata --no-deps --format-version 1 | sed -n 's/.*"name":"pengpilot","version":"\([^"]*\)".*/\1/p')"
target_triple="$(rustc -vV | sed -n 's/^host: //p')"
package="pengpilot-${version}-${target_triple}"
archive="$target_dir/release/$package.tar.gz"
staging="$(mktemp -d)"
trap 'rm -rf -- "$staging"' EXIT

cargo build --locked --release --bin pengpilot

package_dir="$staging/$package"
install -Dm755 "$target_dir/release/pengpilot" "$package_dir/bin/pengpilot"
install -Dm644 resources/linux/com.pengpilot.app.desktop \
  "$package_dir/share/applications/com.pengpilot.app.desktop"
install -Dm644 website/public/app-icon.png \
  "$package_dir/share/icons/hicolor/256x256/apps/com.pengpilot.app.png"
install -Dm644 LICENSE "$package_dir/share/licenses/pengpilot/LICENSE"

mkdir -p "$(dirname "$archive")"
tar -C "$staging" -czf "$archive" "$package"
printf 'Created %s\n' "$archive"
