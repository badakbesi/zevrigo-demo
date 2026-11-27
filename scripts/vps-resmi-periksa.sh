#!/usr/bin/env bash
# vps-resmi-periksa.sh - membandingkan semua VPS resmi Zevrigo (TSK-M76, DEC-47): susunannya harus sama persis. Dijalankan dari
# komputer pencipta (WSL), kunci SSH semua VPS sudah dimuat ke ssh-agent:
#   bash scripts/vps-resmi-periksa.sh [alias-ssh ...]        bawaan: zevrigo-vps zevrigo-eu
# Dibaca dari setiap VPS: ~/zevrigo/VERSI, sidik compose yang terpasang, ID image zevrigo:regtest, versi Docker dan compose,
# layanan profile demo yang harus dan yang jalan, tinggi chain seed dan pengamat. Hasil: tabel, lalu SERAGAM (keluar 0) atau
# BEDA dengan daftar perbedaan (keluar 1). Tidak mengubah apa pun.
set -euo pipefail

daftar=("$@")
if [[ ${#daftar[@]} -eq 0 ]]; then
    daftar=(zevrigo-vps zevrigo-eu)
fi
if ! ssh-add -l > /dev/null 2>&1; then
    echo "BERHENTI: ssh-agent tanpa kunci. Muat dulu: eval \"\$(ssh-agent -s)\"; ssh-add ~/.ssh/<kunci setiap VPS>"
    exit 1
fi
SO=(-o BatchMode=yes -o ConnectTimeout=20 -o ServerAliveInterval=15)
kumpul='cd ~/zevrigo 2> /dev/null || { echo "galat=folder ~/zevrigo tidak ada"; exit 0; }
v() { sed -n "s/^$1=//p" "$2" 2> /dev/null | tail -n 1; }
echo "nama=$(v ZVRG_NAMA_VPS .env)"
echo "profiles=$(v COMPOSE_PROFILES .env)"
echo "versi_commit=$(v commit_demo VERSI)"
echo "versi_compose_yaml=$(v compose_yaml VERSI)"
echo "versi_compose_publik=$(v compose_publik VERSI)"
echo "compose_yaml=$(sha256sum compose.yaml 2> /dev/null | cut -d" " -f1)"
echo "compose_publik=$(sha256sum compose.publik.yaml 2> /dev/null | cut -d" " -f1)"
echo "image=$(docker image inspect --format "{{.Id}}" zevrigo:regtest < /dev/null 2> /dev/null)"
echo "image_simpan=$(docker image ls --format "{{.ID}}" zevrigo < /dev/null 2> /dev/null | sort -u | wc -l)"
echo "docker=$(docker version --format "{{.Server.Version}}" < /dev/null 2> /dev/null)"
echo "compose=$(docker compose version --short < /dev/null 2> /dev/null)"
echo "harus=$(COMPOSE_PROFILES=demo docker compose config --services < /dev/null 2> /dev/null | LC_ALL=C sort | tr "\n" " ")"
echo "jalan=$(docker compose ps --status running --services < /dev/null 2> /dev/null | LC_ALL=C sort | tr "\n" " ")"
echo "tinggi_seed=$(timeout 20 docker compose exec -T node zevrigo-cli info < /dev/null 2> /dev/null | sed -n "s/^tinggi: //p")"
echo "tinggi_pengamat=$(timeout 20 docker compose exec -T node-pengamat zevrigo-cli info < /dev/null 2> /dev/null | sed -n "s/^tinggi: //p")"'

declare -A V
beda=()
for a in "${daftar[@]}"; do
    if ! keluar=$(ssh "${SO[@]}" "$a" "bash -s" <<< "$kumpul" 2>&1); then
        beda+=("$a: SSH gagal ($(tail -n 1 <<< "$keluar"))")
        continue
    fi
    while IFS='=' read -r k isi; do
        if [[ -n $k ]]; then
            V["$a|$k"]=$isi
        fi
    done <<< "$keluar"
done
n() { printf '%s' "${V["$1|$2"]-}"; }
pendek() { local s; s=$(n "$1" "$2"); s=${s#sha256:}; printf '%s' "${s:0:12}"; }

printf '%-22s' "hal"
for a in "${daftar[@]}"; do printf ' | %-26s' "$a"; done
echo
for f in nama profiles versi_commit compose_yaml compose_publik image image_simpan docker compose tinggi_seed tinggi_pengamat; do
    printf '%-22s' "$f"
    for a in "${daftar[@]}"; do
        case $f in
            versi_commit | compose_yaml | compose_publik | image) printf ' | %-26s' "$(pendek "$a" "$f")" ;;
            *) printf ' | %-26s' "$(n "$a" "$f")" ;;
        esac
    done
    echo
done

pertama=${daftar[0]}
for a in "${daftar[@]}"; do
    g=$(n "$a" galat)
    if [[ -n $g ]]; then
        beda+=("$a: $g")
        continue
    fi
    for f in compose_yaml compose_publik image docker compose harus; do
        if [[ -z $(n "$a" "$f") ]]; then
            beda+=("$a: $f tidak terbaca")
        elif [[ $(n "$a" "$f") != "$(n "$pertama" "$f")" ]]; then
            beda+=("$f: $a beda dengan $pertama")
        fi
    done
    if [[ $(n "$a" versi_compose_yaml) != "$(n "$a" compose_yaml)" || $(n "$a" versi_compose_publik) != "$(n "$a" compose_publik)" ]]; then
        beda+=("$a: ~/zevrigo/VERSI tidak cocok dengan compose terpasang (tidak dipasang dengan vps-resmi-pasang.sh)")
    fi
    jalan=" $(n "$a" jalan) "
    for s in $(n "$a" harus); do
        case $jalan in
            *" $s "*) ;;
            *) beda+=("$a: layanan $s tidak jalan") ;;
        esac
    done
    if [[ $(n "$a" image_simpan) =~ ^[0-9]+$ ]] && (($(n "$a" image_simpan) > 2)); then
        beda+=("$a: $(n "$a" image_simpan) image zevrigo tersimpan (aturan: aktif + satu sebelumnya)")
    fi
done
min=""
max=""
for a in "${daftar[@]}"; do
    for f in tinggi_seed tinggi_pengamat; do
        t=$(n "$a" "$f")
        if [[ ! $t =~ ^[0-9]+$ ]]; then
            beda+=("$a: $f tidak terbaca")
            continue
        fi
        if [[ -z $min ]] || ((t < min)); then min=$t; fi
        if [[ -z $max ]] || ((t > max)); then max=$t; fi
    done
done
if [[ -n $min ]] && ((max - min > 2)); then
    beda+=("tinggi chain berselisih $((max - min)) blok (min $min, maks $max)")
fi

echo
if [[ ${#beda[@]} -eq 0 ]]; then
    echo "SERAGAM: ${#daftar[@]} VPS resmi, tinggi $min-$max"
    exit 0
fi
echo "BEDA (${#beda[@]}):"
for b in "${beda[@]}"; do echo "  - $b"; done
exit 1
