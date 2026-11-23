#!/usr/bin/env bash
# peserta-demo-zevrigo.sh - paket peserta jaringan demo Zevrigo (regtest), Linux x86_64.
#
# Menjalankan node regtest yang tersambung sendiri ke seed resmi jaringan demo (--seed-resmi), membuat wallet bila belum ada,
# lalu menambang satu blok setiap ZVRG_JEDA detik (bawaan 120, sama dengan waktu blok target) dengan hadiah ke alamat baru
# wallet itu. Berhenti dengan Ctrl-C: miner berhenti dan node yang dinyalakan skrip ini ikut dimatikan dengan rapi.
#
#   ./peserta-demo-zevrigo.sh
#   ZVRG_JEDA=300 ./peserta-demo-zevrigo.sh          jeda lain (detik)
#   ZVRG_DATA=/folder/lain ./peserta-demo-zevrigo.sh  folder data lain (bawaan ~/.zevrigo)
#
# Koin jaringan demo tidak bernilai. Node hanya membuka RPC di 127.0.0.1 dan tidak menerima sambungan masuk dari luar.
set -u
DIR=$(cd "$(dirname "$0")" && pwd)
DATA=${ZVRG_DATA:-$HOME/.zevrigo}
JEDA=${ZVRG_JEDA:-120}
NODE="$DIR/zevrigo-node"; CLI="$DIR/zevrigo-cli"; MINER="$DIR/zevrigo-miner"
GERBANG=https://demo-gerbang-rpc.zevrigo.online
for b in "$NODE" "$CLI" "$MINER"; do
  [ -x "$b" ] || { echo "Program tidak ditemukan: $b. Jalankan skrip ini dari folder paket yang sudah diekstrak."; exit 1; }
done
case "$JEDA" in '' | *[!0-9]*) echo "ZVRG_JEDA harus jumlah detik, misalnya 120"; exit 2 ;; esac
mkdir -p "$DATA" || exit 1
LOG="$DATA/node.log"
PN=""

tinggi() { "$CLI" info 2>/dev/null | sed -n 's/^tinggi: //p'; }
tinggi_publik() {
  command -v curl > /dev/null || return 0
  curl -s -m 20 -X POST -H 'content-type: application/json' \
    --data '{"jsonrpc":"2.0","id":1,"method":"get_chain_info","params":{}}' "$GERBANG" 2>/dev/null \
    | grep -o '"height":[0-9]*' | head -n 1 | cut -d: -f2
}
berhenti() {
  local kode=${1:-0}
  trap - INT TERM
  echo
  if [ -n "$PN" ] && kill -0 "$PN" 2>/dev/null; then
    echo "Menghentikan node..."
    kill -TERM "$PN" 2>/dev/null
    for _ in $(seq 1 30); do kill -0 "$PN" 2>/dev/null || break; sleep 1; done
  fi
  echo "Selesai. Data tetap di $DATA"
  exit "$kode"
}
trap 'berhenti 0' INT TERM

echo "== Zevrigo jaringan demo (regtest) | folder data: $DATA"
if [ -n "$(tinggi)" ]; then
  echo "Node sudah berjalan di 127.0.0.1:28801; dipakai apa adanya (tidak dimatikan saat skrip berhenti)."
else
  echo "Menyalakan node (log: $LOG)..."
  "$NODE" --regtest --datadir "$DATA" --seed-resmi >> "$LOG" 2>&1 &
  PN=$!
  for _ in $(seq 1 60); do [ -n "$(tinggi)" ] && break; kill -0 "$PN" 2>/dev/null || break; sleep 1; done
  if [ -z "$(tinggi)" ]; then
    echo "Node tidak menyala. Baris terakhir log:"; tail -n 15 "$LOG"; PN=""; exit 1
  fi
fi

echo "Menunggu node tersambung ke jaringan demo dan mengejar chain..."
ok=0; stabil=0; lalu=""
for _ in $(seq 1 120); do
  H=$(tinggi); P=$(tinggi_publik)
  if [ -n "$H" ] && [ -n "$P" ] && [ "$H" -ge "$P" ] && [ "$P" -gt 0 ]; then ok=1; break; fi
  if [ -z "$P" ] && [ -n "$H" ] && [ "$H" -gt 0 ]; then
    if [ "$H" = "$lalu" ]; then stabil=$((stabil + 1)); else stabil=0; fi
    [ "$stabil" -ge 4 ] && { ok=1; break; }
  fi
  lalu=$H
  sleep 5
done
[ "$ok" = 1 ] || { echo "Node belum mengejar chain dalam 10 menit (tinggi lokal ${H:-?}, publik ${P:-?}). Periksa internet, lalu coba lagi."; berhenti 1; }
echo "Node sinkron: tinggi $H."

if [ ! -f "$DATA/regtest/wallet.json" ]; then
  echo
  echo "== Wallet baru"
  echo "Wallet belum ada. Buat password (minimal 8 karakter), lalu CATAT 24 kata yang tampil di kertas dan simpan di tempat aman."
  echo "Tanpa 24 kata itu wallet tidak bisa dipulihkan."
  "$CLI" --datadir "$DATA" wallet-create || { echo "Wallet gagal dibuat."; berhenti 1; }
  read -r -p "Tekan Enter setelah 24 kata dicatat... " _ || true
fi
A=$("$CLI" --datadir "$DATA" address 2>&1) || { echo "Alamat wallet gagal dibaca: $A"; berhenti 1; }
ALAMAT=$(printf '%s\n' "$A" | sed -n 's/^alamat: //p'); PAYOUT=$(printf '%s\n' "$A" | sed -n 's/^payload: //p')
case "$PAYOUT" in *[!0-9a-f]* | "") echo "Payload alamat tidak terbaca: $A"; berhenti 1 ;; esac
[ "${#PAYOUT}" -eq 64 ] || { echo "Payload alamat bukan 64 hex: $PAYOUT"; berhenti 1; }
echo
echo "== Menambang: satu blok setiap $JEDA detik, hadiah ke $ALAMAT"
echo "Lihat saldo: $CLI --datadir $DATA balance"
echo "Hentikan dengan Ctrl-C."
while :; do
  "$MINER" --payout "$PAYOUT" --blocks 1 || echo "Putaran gagal, dicoba lagi sesudah jeda."
  if [ -n "$PN" ] && ! kill -0 "$PN" 2>/dev/null; then
    echo "Node berhenti sendiri. Baris terakhir log:"; tail -n 15 "$LOG"; PN=""; exit 1
  fi
  sleep "$JEDA" &
  wait $!
done
