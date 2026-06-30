#!/usr/bin/env bash
set -euo pipefail

target="${1:-}"
artifact_name="${2:-tiny-retro-racer-macos}"
binary_name="${3:-tiny-retro-racer}"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root_dir="$(cd "${script_dir}/.." && pwd)"
release_dir="${root_dir}/target/release"

if [[ -n "${target}" ]]; then
  release_dir="${root_dir}/target/${target}/release"
fi

binary_source="${release_dir}/${binary_name}"
dist_dir="${root_dir}/dist/${artifact_name}"
app_dir="${dist_dir}/Tiny Retro Racer.app"
contents_dir="${app_dir}/Contents"
macos_dir="${contents_dir}/MacOS"
resources_dir="${contents_dir}/Resources"
bundle_executable="tiny-retro-racer"
version="$(awk -F '"' '/^version = / { print $2; exit }' "${root_dir}/Cargo.toml")"

if [[ -z "${version}" ]]; then
  echo "could not extract version from ${root_dir}/Cargo.toml" >&2
  exit 1
fi

if [[ ! -x "${binary_source}" ]]; then
  echo "missing release binary: ${binary_source}" >&2
  echo "build it first with cargo build --release --bin tiny-retro-racer${target:+ --target ${target}}" >&2
  exit 66
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required to generate the macOS app icon" >&2
  exit 69
fi

if ! command -v iconutil >/dev/null 2>&1; then
  echo "iconutil is required to create the macOS app icon" >&2
  exit 69
fi

rm -rf "${dist_dir}"
mkdir -p "${macos_dir}" "${resources_dir}"

cp "${binary_source}" "${macos_dir}/${bundle_executable}"
chmod +x "${macos_dir}/${bundle_executable}"
cp "${root_dir}/README.md" "${root_dir}/CHANGELOG.md" "${dist_dir}/"

icon_work_dir="$(mktemp -d)"
trap 'rm -rf "${icon_work_dir}"' EXIT

python3 "${script_dir}/generate-macos-icon.py" "${icon_work_dir}/AppIcon.iconset" >/dev/null
iconutil -c icns "${icon_work_dir}/AppIcon.iconset" -o "${resources_dir}/AppIcon.icns"

cat >"${contents_dir}/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>Tiny Retro Racer</string>
  <key>CFBundleExecutable</key>
  <string>${bundle_executable}</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundleIdentifier</key>
  <string>com.lvtd.tiny-retro-racer</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>Tiny Retro Racer</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${version}</string>
  <key>CFBundleVersion</key>
  <string>${version}</string>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.games</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

printf 'APPL????' >"${contents_dir}/PkgInfo"

echo "${app_dir}"
