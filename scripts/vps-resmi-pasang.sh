#!/usr/bin/env bash
# vps-resmi-pasang.sh - memasang susunan VPS resmi (TSK-M76, DEC-47) ke SATU VPS resmi Zevrigo. Susunannya sama di setiap VPS:
# compose.yaml + compose.publik.yaml dari repo ini, profile demo, pilihan per VPS hanya di ~/zevrigo/.env.
# Dijalankan dari repo zevrigo-demo di komputer pencipta (WSL), kunci SSH VPS sudah dimuat ke ssh-agent:
#   bash scripts/vps-resmi-pasang.sh <alias-ssh>          contoh: bash scripts/vps-resmi-pasang.sh zevrigo-eu
# Syarat di VPS: ~/zevrigo/.env (contoh scripts/vps-resmi.env.contoh), ~/.zevrigo.env (token tunnel, izin 600), dan image
# zevrigo:regtest dengan ID yang sama dengan PC (dikirim dulu dengan docker save/load). Langkah:
#   1. compose.yaml dan compose.publik.yaml dari commit HEAD (bukan salinan kerja) disalin ke ~/zevrigo; berkas lama ke
#      ~/zevrigo/sebelum/
#   2. konfigurasi diperiksa, lalu docker compose up -d --no-build (hanya layanan yang berubah yang dibuat ulang)
#   3. semua layanan profile aktif jalan dalam 90 detik, lalu ~/zevrigo/VERSI ditulis
# Gagal di langkah 2-3: berkas lama dipasang lagi dan layanan dinyalakan lagi dengan berkas lama.
set -euo pipefail

alias_vps=${1:-}
if [[ -z $alias_vps || $# -ne 1 ]]; then
    echo "pakai: bash scripts/vps-resmi-pasang.sh <alias-ssh>"
    exit 2
fi
akar=$(git rev-parse --show-toplevel)
cd "$akar"
if [[ -n $(git status --porcelain=v1 -- compose.yaml compose.publik.yaml) ]]; then
    echo "BERHENTI: compose.yaml atau compose.publik.yaml berubah tanpa commit"
    exit 1
fi
if ! ssh-add -l > /dev/null 2>&1; then
    echo "BERHENTI: ssh-agent tanpa kunci. Muat dulu: eval \"\$(ssh-agent -s)\"; ssh-add ~/.ssh/<kunci VPS>"
    exit 1
fi
commit=$(git rev-parse HEAD)
img=$(docker image inspect --format '{{.Id}}' zevrigo:regtest)
SO=(-o BatchMode=yes -o ConnectTimeout=20 -o ServerAliveInterval=15)
kerja=$(mktemp -d)
trap 'rm -rf "$kerja"' EXIT
git show HEAD:compose.yaml > "$kerja/compose.yaml"
git show HEAD:compose.publik.yaml > "$kerja/compose.publik.yaml"

cat > "$kerja/jarak.sh" << 'JARAK'
set -u
stop() { echo "BERHENTI: $1"; exit 1; }
cd ~/zevrigo || stop "folder ~/zevrigo tidak ada"
[ -f .env ] || stop "~/zevrigo/.env tidak ada (contoh scripts/vps-resmi.env.contoh)"
nilai() { sed -n "s/^$1=//p" .env | tail -n 1; }
for k in COMPOSE_FILE COMPOSE_PROFILES ZVRG_NAMA_VPS ZVRG_HOST_SEED ZVRG_DIAM; do
  [ -n "$(nilai "$k")" ] || stop ".env tanpa $k"
done
[ "$(nilai COMPOSE_FILE)" = "compose.yaml:compose.publik.yaml" ] || stop "COMPOSE_FILE harus compose.yaml:compose.publik.yaml"
PROF=$(nilai COMPOSE_PROFILES)
case ",$PROF," in *,demo,*) ;; *) stop "COMPOSE_PROFILES tanpa demo" ;; esac
case ",$PROF," in *,pc,*) stop "COMPOSE_PROFILES berisi pc (profile komputer pencipta)" ;; esac
[ -f ~/.zevrigo.env ] || stop "token tunnel ~/.zevrigo.env tidak ada"
[ "$(stat -c %a ~/.zevrigo.env)" = 600 ] || stop "izin ~/.zevrigo.env bukan 600"
VIMG=$(docker image inspect --format '{{.Id}}' zevrigo:regtest < /dev/null 2> /dev/null)
[ "$VIMG" = "$IMG" ] || stop "image zevrigo:regtest VPS ($VIMG) beda dengan PC ($IMG); kirim image dulu"
cp .env .masuk/.env
( cd .masuk && docker compose config -q < /dev/null ) || stop "konfigurasi compose baru tidak sah"
mkdir -p sebelum
ADA_LAMA=0
for f in compose.yaml compose.publik.yaml VERSI; do
  if [ -f "$f" ]; then cp -p "$f" "sebelum/$f"; ADA_LAMA=1; fi
done
cp .masuk/compose.yaml .masuk/compose.publik.yaml .
rm -rf .masuk
pulih() {
  echo "GAGAL: $1"
  if [ "$ADA_LAMA" = 1 ]; then
    cp -p sebelum/compose.yaml sebelum/compose.publik.yaml . && echo "pemulihan: berkas lama dipasang"
    docker compose up -d --no-build < /dev/null 2>&1 | tail -n 5
  fi
  exit 1
}
docker compose up -d --no-build < /dev/null > /tmp/vps-resmi-up.log 2>&1 || { tail -n 15 /tmp/vps-resmi-up.log; pulih "docker compose up"; }
tail -n 15 /tmp/vps-resmi-up.log
HARUS=$(docker compose config --services < /dev/null | LC_ALL=C sort | tr '\n' ' ')
i=0; KURANG=$HARUS
while [ "$i" -lt 18 ]; do
  sleep 5; i=$((i + 1))
  JALAN=" $(docker compose ps --status running --services < /dev/null | LC_ALL=C sort | tr '\n' ' ') "
  KURANG=""
  for s in $HARUS; do case "$JALAN" in *" $s "*) ;; *) KURANG="$KURANG $s" ;; esac; done
  [ -z "$KURANG" ] && break
done
[ -z "$KURANG" ] || pulih "layanan tidak jalan sesudah 90 detik:$KURANG"
echo "layanan jalan: $HARUS"
{
  echo "nama=$(nilai ZVRG_NAMA_VPS)"
  echo "commit_demo=$COMMIT"
  echo "compose_yaml=$(sha256sum compose.yaml | cut -d' ' -f1)"
  echo "compose_publik=$(sha256sum compose.publik.yaml | cut -d' ' -f1)"
  echo "image=$IMG"
  echo "docker=$(docker version --format '{{.Server.Version}}' < /dev/null)"
  echo "compose=$(docker compose version --short < /dev/null)"
  echo "profiles=$PROF"
  echo "dipasang=$(date -Is)"
} > VERSI
echo "VERSI:"; sed 's/^/  /' VERSI
echo "SELESAI"
JARAK

echo "VPS $alias_vps: commit $commit, image $img"
ssh "${SO[@]}" "$alias_vps" 'mkdir -p ~/zevrigo/.masuk'
scp -q "${SO[@]}" "$kerja/compose.yaml" "$kerja/compose.publik.yaml" "$alias_vps:zevrigo/.masuk/"
ssh "${SO[@]}" "$alias_vps" "IMG=$img COMMIT=$commit bash -s" < "$kerja/jarak.sh"
