#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${1:-$ROOT/dist}"
WORK_DIR="${2:-$ROOT/target/live-build}"

if [[ "$(uname -s)" != Linux ]]; then
  echo "Build this image on Debian or Ubuntu Linux (live-build needs Linux root tools)." >&2
  exit 2
fi
if [[ "$(uname -m)" != x86_64 ]]; then
  echo "The current ISO recipe targets amd64 and must be built on an x86_64 host." >&2
  exit 2
fi
for tool in cargo lb; do
  command -v "$tool" >/dev/null || { echo "Missing required tool: $tool" >&2; exit 2; }
done

cargo build --release --locked --manifest-path "$ROOT/Cargo.toml" --bin rustblocks
mkdir -p "$OUT_DIR" "$WORK_DIR"
cd "$WORK_DIR"
rm -f config/binary config/bootstrap config/chroot config/common config/source
lb config \
  --mode debian \
  --distribution trixie \
  --architectures amd64 \
  --binary-images iso-hybrid \
  --debian-installer false \
  --archive-areas "main contrib non-free non-free-firmware" \
  --bootappend-live "boot=live components username=player quiet"

mkdir -p config/package-lists config/includes.chroot/usr/local/bin \
  config/includes.chroot/etc/skel/.config/autostart
cat > config/package-lists/rustblocks.list.chroot <<'EOF'
task-xfce-desktop
live-boot
live-config
live-config-systemd
xfce4
dbus-x11
libasound2t64
libgl1
libx11-6
libxcursor1
libxi6
libxinerama1
libxrandr2
libxkbcommon0
EOF
install -m 0755 "$ROOT/target/release/rustblocks" config/includes.chroot/usr/local/bin/rustblocks
cat > config/includes.chroot/etc/skel/.config/autostart/rustblocks.desktop <<'EOF'
[Desktop Entry]
Type=Application
Name=RustBlocks
Exec=/usr/local/bin/rustblocks
Terminal=false
X-GNOME-Autostart-enabled=true
EOF
lb build
image=$(find "$WORK_DIR" -maxdepth 1 -type f -name '*.iso' -print -quit)
if [[ -z "$image" ]]; then
  echo "live-build finished without producing an ISO" >&2
  exit 1
fi
install -m 0644 "$image" "$OUT_DIR/rustblocks-live-amd64.iso"
if command -v sha256sum >/dev/null; then
  (cd "$OUT_DIR" && sha256sum rustblocks-live-amd64.iso > rustblocks-live-amd64.iso.sha256)
fi
echo "Created $OUT_DIR/rustblocks-live-amd64.iso"
