#!/usr/bin/env bash
# Pemasangan pertama Zevrigo Node di VPS (aplikasi Zevrigo Node Android, DEC-46, Q199a). Dikirim aplikasi lewat SSH ke
# ~/.cache/demo-zevrigo-pasang.sh lalu dijalankan sebagai root (sudo): bash <berkas> <sha256 token pengendali> <sha256 paket>.
# Mengunduh paket resmi demo-zevrigo-vps, mencocokkan SHA-256 yang dikenal aplikasi, lalu demo-zevrigo-kendali pasang.
# Menghapus dirinya sendiri. Keluar 0 bila berhasil; selain itu baris terakhir stderr "galat: ...".
set -u
export LC_ALL=C PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
exec < /dev/null
SENDIRI=$0
trap 'rm -f "$SENDIRI"' EXIT
RILIS=https://github.com/badakbesi/zevrigo-demo/releases/download/demo-zevrigo-vps
PAKET=demo-zevrigo-vps-linux-x86_64.tar.gz
TSHA=${1:-}
PSHA=${2:-}
galat() { echo "galat: $*" >&2; exit 1; }
[ "$(id -u)" = 0 ] || galat "pemasangan butuh root"
[[ "$TSHA" =~ ^[0-9a-f]{64}$ ]] && [[ "$PSHA" =~ ^[0-9a-f]{64}$ ]] || galat "argumen pemasangan tidak sah"
U=${SUDO_USER:-root}
[[ "$U" =~ ^[a-z_][a-z0-9_-]{0,31}$ ]] || galat "nama pengguna SSH tidak sah"
if command -v curl > /dev/null; then unduh() { curl -fsSL --retry 3 --connect-timeout 30 -o "$2" "$1"; }
elif command -v wget > /dev/null; then unduh() { wget -q -T 30 -t 3 -O "$2" "$1"; }
else galat "curl atau wget tidak ada di VPS"; fi
K=$(mktemp -d /tmp/demo-zevrigo-pasang.XXXXXX) || galat "folder sementara tidak bisa dibuat"
trap 'rm -rf "$K"; rm -f "$SENDIRI"' EXIT
unduh "$RILIS/SHA256SUMS.txt" "$K/SHA256SUMS.txt" || galat "SHA256SUMS.txt tidak bisa diunduh dari GitHub"
grep -qxF "$PSHA  $PAKET" "$K/SHA256SUMS.txt" || galat "rilis resmi tidak memuat paket yang dikenal aplikasi ini (perbarui aplikasi)"
unduh "$RILIS/$PAKET" "$K/$PAKET" || galat "paket tidak bisa diunduh dari GitHub"
[ "$(sha256sum "$K/$PAKET" | cut -d' ' -f1)" = "$PSHA" ] || galat "SHA-256 paket unduhan tidak cocok"
tar -xzf "$K/$PAKET" -C "$K" --no-same-owner || galat "paket tidak bisa diekstrak"
[ -x "$K/demo-zevrigo-vps/demo-zevrigo-kendali" ] || galat "isi paket tidak dikenali"
if [ "$U" = root ]; then
  "$K/demo-zevrigo-vps/demo-zevrigo-kendali" pasang --token-sha256 "$TSHA" --sha256 "$PSHA"
else
  "$K/demo-zevrigo-vps/demo-zevrigo-kendali" pasang --pengguna-ssh "$U" --token-sha256 "$TSHA" --sha256 "$PSHA"
fi
