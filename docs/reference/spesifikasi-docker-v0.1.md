# Spesifikasi Paket Docker Zevrigo Coin (ZVRG) v0.1: node, explorer, dan antarmuka web untuk regtest

> Diperbarui: 27 November 2026 (Bagian 7 dan 7.1, VPS resmi ketiga `zevrigo-roy`, Q207c; sebelumnya 23 November 2026, Bagian 7, seed resmi bawaan dan pertukaran alamat, Q191) · Pemilik: pencipta · Ditulis: Quick · Task: TSK-M33, TSK-M44, TSK-M45, TSK-M46, TSK-M47, TSK-M53, TSK-M65, TSK-M76 · Status: 🟢 v0.1
> Lokasi di repo: `docs/reference/spesifikasi-docker-v0.1.md`
> **Sejak 18 November 2026 (DEC-39, Q151):** paket Docker ada di repo `zevrigo-demo` (`Dockerfile`, `Dockerfile.dockerignore`,
> `compose.yaml`, `compose.publik.yaml`), dengan konteks build folder induk `zevrigo_lab`. Jalur lama di dokumen ini:
> `apps/explorer` = repo `zevrigo-explorer`, `apps/demo` = repo ini, `apps/blog` = repo `zevrigo-blog`; `crates/`,
> `apps/node`, `apps/cli`, `apps/miner`, `tests/`, dan `vendor/` = repo inti `zevrigo`.

| Status | Ringkasan |
|---|---|
| 🟢 Dikunci | Satu image `zevrigo:regtest` (node, explorer, miner, CLI, layanan demo, antarmuka web) dari image dasar resmi yang dikunci digest, pengguna biasa uid 10001, data di volume `/data`. `compose.yaml` menjalankan node + explorer; explorer hanya terbuka di `127.0.0.1:28802` komputer host. `compose.publik.yaml` membuka explorer dan seed P2P WebSocket regtest ke publik lewat Cloudflare Tunnel, dengan miner berkala satu blok setiap 120 detik (Bagian 7) |
| ⏳ Ditunda | Paket testnet (sesudah genesis testnet, TSK-F06), mainnet, seed testnet, image untuk arsitektur lain (arm64), build yang bisa diulang bit demi bit (TSK-F10) |
| ➡️ Langkah berikutnya | Paket testnet: node `--testnet`, explorer `--testnet`, tunnel ke port 18802. Batas laju per pengunjung di belakang tunnel selesai (TSK-M43) |

Paket Docker hanya membungkus program yang sudah ada. Tidak ada aturan chain, kunci, atau data baru di dalamnya.

---

## 1. Image dasar (DEC-10 bagian 9)

| Tahap | Image | Digest indeks |
|---|---|---|
| Build Rust | `rust:1.99.0-slim-trixie` | `sha256:2752b332db73fdbb7dc576f06c82ed1f312005784ef913d7e04a28f5f55dc581` |
| Build web | `node:24.22.0-trixie-slim` | `sha256:a6155384446670e9719bdc4fd0ebe4d4b29fb32a8d6aa523da3547a6cfbcf22c` |
| Runtime | `debian:trixie-slim` (Debian 13) | `sha256:f6b08b09a05725f9431502d3b5714f5c4576e82c1a84fa1debe0c24a7e0c3ec4` |

- Digest dibaca dari registry Docker Hub dengan `docker buildx imagetools inspect` (Q93, 3 November 2026).
- Rust 1.99.0 sama dengan `rust-toolchain.toml`; Node.js 24.22.0 sama dengan lingkungan WSL pencipta.
- Mengganti image dasar = mengganti digest di `Dockerfile` dan di tabel ini, lewat skrip gerbang.
- Tahap build Rust memasang `cmake`, `make`, dan `g++` dari repositori Debian untuk kode C++ RandomX (DEC-01, DEC-10 bagian 10).
  Versi paket Debian belum dikunci; dikunci bersama compiler C++ di TSK-F10.

## 2. Isi image

| Bagian | Isi |
|---|---|
| Program | `/usr/local/bin/zevrigo-node`, `zevrigo-explorer`, `zevrigo-miner`, `zevrigo-cli`, `zevrigo-demo` (MOD-18, sejak TSK-M53) (`cargo build --release --locked`) |
| Antarmuka web | `/usr/share/zevrigo/web` (hasil `npm ci --ignore-scripts` dan `next build`, static export) |
| Pengguna | `zevrigo` (uid dan gid 10001), bukan root |
| Data | Volume `/data`: chain `regtest/chain.redb`, indeks `regtest/explorer.sqlite`, wallet `regtest/wallet.json` |
| Perintah bawaan | `zevrigo-node --regtest --datadir /data` |

Konteks build adalah folder induk `zevrigo_lab`; yang masuk hanya daftar izin di `Dockerfile.dockerignore`: dari repo inti
`Cargo.toml`, `Cargo.lock`, `crates/`, `apps/node`, `apps/cli`, `apps/miner`, `tests/`, dan `vendor/` (salinan RandomX dengan
salt ZVRG, DEC-01); dari `zevrigo-explorer` `Cargo.toml`, `Cargo.lock`, `src/`, dan `web/`; dari `zevrigo-demo` `Cargo.toml`,
`Cargo.lock`, dan `src/`; dari `zevrigo-blog` seluruh repo; selalu tanpa `node_modules`, `.next`, `out`, `target`, dan `.git`.
`rust-toolchain.toml` dan `.cargo/` tidak disalin (tahap build Rust memeriksanya). Program Rust dibangun dari tiga workspace
(repo inti, zevrigo-explorer, zevrigo-demo), masing-masing dengan `Cargo.lock` sendiri dan `--locked`.

## 3. Compose (`compose.yaml`)

| Layanan | Isi |
|---|---|
| `node` | Node regtest; satu-satunya layanan dengan port di host: `127.0.0.1:28802` (milik explorer) |
| `explorer` | `zevrigo-explorer --http-bind 0.0.0.0 --web-dir /usr/share/zevrigo/web`, berbagi namespace jaringan dengan `node` |
| `miner` | Profil `miner`, payload dari variabel `ZVRG_PAYOUT`, berbagi namespace jaringan dengan `node` |

- Karena berbagi namespace, explorer, miner, dan CLI menjangkau RPC node di `127.0.0.1:28801` di dalam container. RPC
  dan P2P node tidak pernah diterbitkan ke host.
- Explorer di dalam container mendengar di `0.0.0.0:28802` (opsi `--http-bind`, Spesifikasi Explorer v0.1 Bagian 6);
  Docker hanya meneruskannya ke `127.0.0.1:28802` komputer host. Membuka explorer ke internet tetap wajib lewat reverse
  proxy HTTPS (belum dikerjakan). Explorer memakai batas laju bawaan (`--api-rate 10`, TSK-M40); karena semua koneksi
  dari host masuk lewat penerus port Docker, batas itu berlaku bersama untuk semua tab browser di komputer host.
- Data tetap di volume `zevrigo_zvrg-data` sesudah `docker compose down`; `docker compose down -v` menghapusnya.

## 4. Perintah

```bash
docker compose build                                          # atau: make docker-build
docker compose up -d                                          # node + explorer, buka http://127.0.0.1:28802/
docker compose exec node zevrigo-cli --datadir /data wallet-create
docker compose exec node zevrigo-cli --datadir /data address
ZVRG_PAYOUT=<payload hex 64> docker compose --profile miner up -d miner
docker compose logs -f explorer
docker compose down
```

## 5. Gerbang (Q94)

Image dibangun dari repo; setiap program menjawab `--version`; pengguna image uid 10001; antarmuka web ada; compose
menjalankan node + explorer; explorer di host menyajikan halaman dengan kebijakan keamanan konten, `/api/v1/address/…`
dan `/api/v1/allocation` `404`; miner di dalam container menambang 3 blok dan explorer mengindeksnya; RPC node tidak
bisa dijangkau dari host; `docker compose down -v` membersihkan container dan volume uji.

## 6. Keputusan

| ID | Keputusan | Status |
|---|---|---|
| DEC-10 bagian 9 | Image dasar Docker Bagian 1, dikunci digest, hanya untuk paket Docker | 🟢 3 November 2026 (konsekuensi TSK-M33, pilihan pencipta) |
| DEC-10 bagian 9 (tambahan) | Image resmi `cloudflare/cloudflared` 2026.10.7, digest `sha256:4955524aeaa379e154023d5a027c250f7b35041a2e7e950cf13f144bc9b9c5ba` (Q120), hanya di `compose.publik.yaml` | 🟢 11 November 2026 (pilihan pencipta: explorer publik sementara lewat Cloudflare) |

## 7. Explorer dan seed P2P publik lewat Cloudflare Tunnel (`compose.publik.yaml`, TSK-M44, TSK-M45)

Explorer, seed P2P, gerbang RPC, dan faucet dibuka ke internet lewat Cloudflare Tunnel dengan domain resmi `zevrigo.online`
(DEC-42; domain lama `ratuaurion.store` lewat tunnel kedua, tetap jalan selama peralihan), dulu dari komputer
pencipta, sejak 17 November 2026 dari VPS resmi dengan susunan yang sama di setiap VPS (Bagian 7.1). `cloudflared` hanya membuat koneksi keluar ke Cloudflare; tidak ada port router yang dibuka, dan
alamat IP komputer pencipta tidak terlihat oleh pengunjung. HTTPS dan perlindungan DDoS di sisi Cloudflare.

| Hal | Isi |
|---|---|
| Pemakaian | Dipakai bersama `compose.yaml` (`-f compose.yaml -f compose.publik.yaml`), project `zevrigo-publik` (volume sendiri `zevrigo-publik_zvrg-data`) |
| `node` | Port host dicabut (`ports: !reset []`); RPC dan P2P TCP tetap hanya di dalam namespace. `--p2p-ws-port 28803` (TSK-M45): P2P WebSocket `/ws` di `127.0.0.1:28803` dalam namespace, hanya dicapai lewat `cloudflared`. `--p2p-publik /dns4/$ZVRG_HOST_SEED/tcp/443/tls/sni/$ZVRG_HOST_SEED/ws` (Q191, TSK-M75; nama host seed VPS itu dari `~/zevrigo/.env`, bawaan `demo-seed-p2p.zevrigo.online`): alamat resmi itu diumumkan ke rekan lewat protokol `addr`. `--seed-resmi` (TSK-M76): seed resmi saling tersambung; alamat bawaan milik seed itu sendiri tidak menghasilkan rekan. Batas 384 MiB, 1 CPU |
| `explorer` | `--web-dir /usr/share/zevrigo/web --trust-proxy --faucet-origin https://demo-faucet.zevrigo.online` (TSK-M54: halaman `/faucet/` boleh memanggil faucet publik, DEC-42), mendengar di `127.0.0.1:28802` (bawaan) di dalam namespace node; tidak bisa dijangkau dari host maupun container lain. Batas 256 MiB, 0,5 CPU |
| `cloudflared`, `cloudflared-zevrigo` | Image resmi dikunci digest (Bagian 6), `tunnel --no-autoupdate run`, berbagi namespace jaringan dengan `node`, sehingga sampai ke explorer dan WebSocket P2P lewat loopback. `cloudflared-zevrigo` (profile `demo`) menjalankan tunnel Cloudflare VPS itu (domain `zevrigo.online`, DEC-42; setiap VPS resmi punya tunnel sendiri); `cloudflared` (profile `lama`, sementara, hanya VPS resmi pertama) tunnel lama `ratuaurion.store` sampai dihentikan (DEC-42, paling cepat 23 Desember 2026) |
| `gerbang-rpc` | TSK-M53 (DEC-36): `zevrigo-demo rpc --trust-proxy` di namespace `node` (RPC node lewat `127.0.0.1:28801`), mendengar `127.0.0.1:28804`, hanya dicapai lewat `cloudflared`. Meneruskan hanya `get_chain_info`, `get_block_hash`, `get_block`, `get_mempool`, `scan_utxos`, `scan_mempool`, `send_raw_transaction`; `get_block_template`, `submit_block`, dan metode lain `-32601`; batch `-32600`; badan di atas 256 KiB `413`; batas laju 20 per detik per pengunjung (`CF-Connecting-IP`). Batas 256 MiB, 0,5 CPU, `restart: unless-stopped` |
| `miner-berkala` | TSK-M46, TSK-M47: image `zevrigo:regtest`, `/bin/sh -c`: tunggu 30 detik, lalu ulangi `zevrigo-miner --blocks 1` dan `sleep $ZVRG_JEDA` (bawaan 120 detik = waktu blok target). Berbagi namespace jaringan dengan `node-penambang` (RPC lewat `127.0.0.1:28801` node itu), volume `zvrg-penambang` dipasang baca saja. Hadiah ke payload hex 64 di `/data/payout-penambang` (ditulis operator dari wallet penambang); selama file itu belum ada atau tidak sah, ke kunci NUMS `50929b74…3ac0` yang tidak bisa dibelanjakan. Batas 256 MiB, 0,5 CPU, `init: true`, `restart: unless-stopped` |
| `node-penambang`, `node-treasury`, `node-faucet`, `node-pengamat` | TSK-M47 (DEC-34): node operator jaringan demo, container dan volume sendiri (`zvrg-penambang`, `zvrg-treasury`, `zvrg-faucet`, `zvrg-pengamat`), tanpa port host, `--seed-resmi` (Q191, DEC-43: seed resmi bawaan node, alamat Bagian 7 di bawah; sebelumnya `--connect` dengan `ZVRG_SEED`), lewat internet dan Cloudflare seperti peserta, memakai 4 dari 32 slot masuk seed. Batas 256 MiB dan 0,5 CPU per node (Q125: node seed sekitar 10 MiB, node baru sekitar 2 MiB dan sinkron dalam 8 detik pada tinggi 121; volume data seed 4,4 MB). Wallet operator dibuat pencipta sendiri dengan CLI di container (`wallet-create`, password diketik di terminal), file di `/data/regtest/wallet.json` volume itu |
| `faucet` | TSK-M54 bagian B: `zevrigo-demo faucet --datadir /data --http-bind 0.0.0.0 --trust-proxy-host node --allow-origin https://demo-explorer.zevrigo.online --allow-origin https://demo-explorer-eu.zevrigo.online --allow-origin https://demo-explorer-roy.zevrigo.online --allow-origin https://zevrigo.ratuaurion.store` (halaman explorer setiap VPS resmi dan host lama, DEC-42) di namespace `node-faucet` (RPC node itu lewat `127.0.0.1:28801`), volume sendiri `zvrg-faucet-dompet` (setiap VPS resmi punya wallet panas sendiri; frasa wallet panas `faucet-frasa.txt` izin 600, batas harian `faucet-batas.json`). Mendengar `0.0.0.0:28805` di jaringan Docker tanpa port host; `cloudflared` menjangkaunya lewat `http://node-faucet:28805`. `CF-Connecting-IP` hanya dipercaya dari alamat layanan `node` (namespace `cloudflared`), di-resolve ulang tiap 60 detik. Diisi ulang bertahap dari kantong Faucet treasury dengan `zevrigo-cli send --dari` (TSK-M63). Batas 256 MiB, 0,5 CPU, `restart: unless-stopped` |
| Token | `${HOME}/.zevrigo.env` (tunnel Cloudflare VPS itu; di compose tidak wajib agar profil `pc` tetap terbaca tanpa berkas ini) dan `${HOME}/.zevrigo-tunnel.env` (tunnel lama, hanya VPS resmi pertama) di luar repo, izin `600`, satu baris `TUNNEL_TOKEN=` diikuti token dari dasbor Cloudflare Zero Trust (Networks, Tunnels, Install and run, pilihan Docker). Token tidak pernah ditulis ke repo, log skrip, atau chat |
| Public hostname | Diatur di dasbor Cloudflare (tab Routes, Published application), semuanya tipe **HTTP** (bukan HTTPS: TLS diputus Cloudflare). Tunnel `zevrigo` (DEC-42): `demo-explorer.zevrigo.online` → `localhost:28802` (explorer), `demo-seed-p2p.zevrigo.online` → `localhost:28803` (seed P2P WebSocket), `demo-gerbang-rpc.zevrigo.online` → `localhost:28804` (gerbang RPC, TSK-M53), `demo-faucet.zevrigo.online` → `node-faucet:28805` (faucet, TSK-M54, lewat jaringan Docker, bukan loopback). Setiap VPS resmi berikutnya punya tunnel sendiri dengan route yang sama dan nama host berakhiran lokasi: VPS Eropa (tunnel `zevrigo-eu`) `demo-explorer-eu`, `demo-seed-p2p-eu`, `demo-gerbang-rpc-eu`, `demo-faucet-eu` `.zevrigo.online`; VPS ketiga (tunnel `zevrigo-roy`) `demo-explorer-roy`, `demo-seed-p2p-roy`, `demo-gerbang-rpc-roy`, `demo-faucet-roy` `.zevrigo.online` (TSK-M76). Tunnel lama, tujuan sama: `zevrigo`, `zevrigo-seed`, `zevrigo-rpc`, `zevrigo-faucet` `.ratuaurion.store`; subdomain lain di domain itu milik proyek lain. Blog tidak lagi di VPS (Q186): situs utama `zevrigo.online` di Cloudflare. Satu tingkat subdomain, tercakup sertifikat Universal Cloudflare. Testnet nanti `localhost:18802` dan `localhost:18803` |
| Bergabung ke seed | `zevrigo-node --regtest --datadir <folder> --seed-resmi` (seed resmi bawaan, DEC-43), atau `--connect /dns4/demo-seed-p2p.zevrigo.online/tcp/443/tls/sni/demo-seed-p2p.zevrigo.online/ws/p2p/<PeerId seed>` (alamat lama `zevrigo-seed.ratuaurion.store` masih menjawab selama peralihan). PeerId seed tercetak di log node (`p2p-ws: …`) dan tetap selama volume data sama. Keepalive `status` 60 detik menjaga WebSocket tetap hidup (Cloudflare menutup yang diam sekitar 100 detik) |

Keamanan `--trust-proxy` (Spesifikasi Explorer v0.1 Bagian 6): yang tersambung ke explorer lewat loopback hanya
`cloudflared` dan proses di namespace yang sama (node dan miner, yang tidak memakai HTTP explorer). Siapa pun yang memegang
token bisa menjalankan tunnel ini; jika token bocor, token diganti di dasbor Cloudflare.

Seed regtest (pilihan pencipta, DEC-33) hanya bukti P2P: dengan PoW dev siapa pun yang tersambung bisa menambang instan.
Semua peserta masuk lewat `cloudflared`, jadi tampak dari 127.0.0.1: batas 32 koneksi masuk dibagi bersama dan blokir
hanya per `PeerId`.

```bash
docker compose -f compose.yaml -f compose.publik.yaml build node
docker compose -f compose.yaml -f compose.publik.yaml up -d
docker compose -f compose.yaml -f compose.publik.yaml logs -f cloudflared
docker compose -f compose.yaml -f compose.publik.yaml logs -f miner-berkala
docker compose -f compose.yaml -f compose.publik.yaml logs -f gerbang-rpc
docker compose -f compose.yaml -f compose.publik.yaml logs faucet | grep '^alamat faucet:'      # alamat isi ulang faucet
docker compose -f compose.yaml -f compose.publik.yaml logs --no-log-prefix node | grep '^p2p-ws:'
docker compose -f compose.yaml -f compose.publik.yaml logs --no-log-prefix node-penambang | grep '^p2p:'   # PeerId node operator
ZVRG_JEDA=60 docker compose -f compose.yaml -f compose.publik.yaml up -d --no-deps miner-berkala   # ganti jeda
docker compose -f compose.yaml -f compose.publik.yaml stop miner-berkala                           # hentikan miner saja
docker compose -f compose.yaml -f compose.publik.yaml down
```

Gerbang (Q121): token ada dengan izin `600` (isinya tidak dibaca ke log), konfigurasi compose sah tanpa port host, image
dibangun ulang dengan `--trust-proxy`, `cloudflared` mencatat koneksi tunnel, miner di dalam namespace menambang 5 blok,
lalu dari luar `https://zevrigo.ratuaurion.store/api/v1/status` menjawab `network` regtest dengan tinggi indeks paling
sedikit 5 dan halaman `/` menjawab `200`.

Gerbang seed (Q122): node mencetak `p2p-ws:`; node di container lain (jaringan Docker biasa, lewat internet) tersambung
ke `zevrigo-seed.ratuaurion.store` dengan alamat di atas, mengejar tinggi seed, menerima blok baru lewat siaran, dan masih
tersambung sesudah 150 detik; explorer publik tetap menjawab.

Gerbang miner berkala (Q123): konfigurasi compose sah dengan `miner-berkala` di namespace node tanpa port host; layanan
dinyalakan tanpa menyalakan ulang node, explorer, atau cloudflared; dalam 300 detik tinggi seed naik paling sedikit 2 blok
dan log mencatat `blok diterima`; explorer publik `https://zevrigo.ratuaurion.store/api/v1/status` mengikuti tinggi baru;
kebijakan restart `unless-stopped`.

Gerbang lima node (Q126): konfigurasi compose sah; 4 node operator tanpa port host dengan batas 256 MiB dan 0,5 CPU dan
`--connect` ke seed publik; `miner-berkala` di namespace `node-penambang` dengan volume baca saja; node seed, explorer, dan
cloudflared tidak dinyalakan ulang; dalam 180 detik setiap node operator mencatat `rekan terhubung` dan mencapai tinggi seed,
dan seed mencatat paling sedikit 4 rekan masuk baru; dalam 330 detik tinggi seed naik paling sedikit 2 blok dari
`miner-berkala` yang baru (payout NUMS selama `/data/payout-penambang` belum ada) dan semua node operator mengikuti; explorer
publik mengikuti tinggi baru.

Gerbang RPC (Q127): Cargo.lock hanya bertambah entri `zevrigo-demo`; `make check-all` lolos; image dibangun ulang dan
`zevrigo-demo --version` menjawab; konfigurasi compose sah dengan `gerbang-rpc` di namespace node, tanpa port host, `rpc
--trust-proxy`; `gerbang-rpc` dinyalakan tanpa menyalakan ulang container lain; di namespace seed `zevrigo-cli --rpc-url
http://127.0.0.1:28804 info` menjawab tinggi yang sama dengan node; dari internet `https://zevrigo-rpc.ratuaurion.store`
menjawab `get_chain_info` (`network` regtest) dan `get_block_hash` tinggi 0 (`network_id` regtest), menjawab
`get_block_template` dan `submit_block` dengan `-32601`, batch dengan `-32600`, `GET` dengan `405`, dan badan 262.145 byte
(batas + 1) dengan `413`; explorer publik tetap menjawab.

Gerbang peluncuran tertutup (Q133, TSK-F11 bagian 1): image dibangun ulang (alat `upacara`, mode `faucet`, `send --dari`)
sebelum layanan dihentikan; semua container dihentikan, kelima volume dicadangkan (tar) di luar repo, lalu `chain.redb`,
indeks explorer, dan tembolok riwayat wallet dihapus (`p2p_key`, `wallet.json`, `payout-penambang` tetap); seed dinyalakan
sendiri tanpa tunnel (tidak ada rekan), 99 blok ke alamat penambang, rencana upacara sama dengan `treasury_kantong.txt`,
upacara dikirim dan masuk blok 100, `scan_utxos` di seed: 8 kantong tepat satu UTXO dengan jumlah persis dan `G` habis;
lalu explorer, gerbang RPC, dan tunnel dibuka, 4 node operator sinkron ke chain baru, saldo wallet treasury = 10.000.000 ZVRG
− biaya dengan 8 UTXO, faucet menjawab dari namespace seed (`http://node-faucet:28805`, saldo 0 sebelum isi ulang),
`miner-berkala` melanjutkan dari blok 101, dan dari internet gerbang RPC menunjukkan 8 kantong yang sama.

### 7.1 Susunan VPS resmi seragam (TSK-M76, DEC-47; sebelumnya dua mesin TSK-M65, DEC-38)

| Hal | Isi |
|---|---|
| Aturan | **Setiap VPS resmi Zevrigo memakai susunan yang sama persis**, tanpa versi per lokasi (pencipta, 27 November 2026). Hanya untuk VPS resmi; operator lain bebas memakai susunannya sendiri |
| Profile | `demo` = susunan VPS resmi: `node` (seed), `node-pengamat`, `explorer`, `gerbang-rpc`, `node-faucet` + `faucet`, `penambang-cadangan`, `cloudflared-zevrigo`. `pc` = komputer pencipta: `node-penambang` + `miner-berkala`, `node-treasury` (kunci kantong treasury tidak pernah di VPS). `lama` = `cloudflared` tunnel lama, sementara, hanya VPS resmi pertama (DEC-42) |
| Folder | `~/zevrigo` berisi `compose.yaml` dan `compose.publik.yaml` dari satu commit repo ini (disalin `scripts/vps-resmi-pasang.sh`, bukan disunting di VPS), `.env`, `VERSI`, dan `sebelum/` (berkas pemasangan sebelumnya). Project `zevrigo-publik`, nama volume sama di setiap VPS (`zevrigo-publik_zvrg-data`, `_zvrg-pengamat`, `_zvrg-faucet`, `_zvrg-faucet-dompet`) |
| `.env` | Satu-satunya tempat pilihan per VPS (contoh `scripts/vps-resmi.env.contoh`): `COMPOSE_FILE=compose.yaml:compose.publik.yaml`, `COMPOSE_PROFILES=demo` (VPS pertama `demo,lama` selama tunnel lama jalan), `ZVRG_NAMA_VPS`, `ZVRG_HOST_SEED` (nama host seed VPS itu, karena setiap seed punya PeerId sendiri), `ZVRG_DIAM` (jeda diam penambang cadangan, berbeda per VPS agar tidak menambang bersamaan: VPS pertama 300, Eropa 420, ketiga 540, ...). Perintah sama di setiap VPS: `cd ~/zevrigo && docker compose up -d --no-build`. `scripts/vps-resmi-pasang.sh` memakai `--force-recreate` (semua layanan dibuat ulang; layanan di namespace `node` harus ikut bila `node` dibuat ulang) |
| Token | `~/.zevrigo.env` (izin 600, `TUNNEL_TOKEN=`), tunnel Cloudflare sendiri per VPS dengan route yang sama (Bagian 7, Public hostname) |
| Batas sumber daya | Sama di setiap VPS: seed 384 MiB / 1 CPU; explorer, node lain, gerbang RPC, faucet 256 MiB / 0,5 CPU; penambang cadangan 128 MiB / 0,5 CPU |
| Image | Dibangun di PC (`docker compose … build node`), dikirim `docker save` + `scp` + `docker load`; ID image sama di semua mesin. VPS tidak membangun (`--no-build`); `cloudflared` ditarik VPS dengan digest yang dikunci. Di setiap VPS disimpan image aktif dan satu image sebelumnya (tag `zevrigo:regtest-sebelumnya`) |
| `VERSI` | Ditulis `scripts/vps-resmi-pasang.sh` setiap pemasangan: `nama`, `commit_demo`, sidik `compose_yaml` dan `compose_publik`, `image`, versi `docker` dan `compose`, `profiles`, `dipasang` |
| Pemeriksaan | `scripts/vps-resmi-periksa.sh [alias-ssh ...]` dari PC (bawaan: ketiga VPS resmi `zevrigo-vps zevrigo-eu zevrigo-roy`): membandingkan sidik compose, image, versi Docker/compose, layanan profile `demo` yang jalan, `VERSI`, jumlah image tersimpan, dan tinggi chain seed + pengamat (selisih paling banyak 2). Hasil `SERAGAM` atau `BEDA` dengan daftarnya |
| VPS dasar | Ubuntu 26.04 x86_64 KVM, NAT (hanya SSH yang diteruskan), tanpa IPv6. User `zvrg` (sudo, docker), login SSH hanya dengan kunci (`/etc/ssh/sshd_config.d/0-zevrigo.conf`), `ufw` masuk hanya SSH, swap 2 GiB, pembaruan keamanan otomatis, NTP. Docker Engine dari repository resmi Docker, log `json-file` 10 MB × 3. VPS resmi: pertama (Indonesia, Q145, 17 November 2026), Eropa (Q203, 26 November 2026), ketiga `zevrigo-roy` (Q207, 27 November 2026; 1 vCPU, 1 GB RAM, susunan seragam Q207c) |
| Akibat | PC mati: setiap VPS tetap menjawab explorer, gerbang RPC, faucet, dan seed; `penambang-cadangan` menambang bila chain diam `ZVRG_DIAM` detik. Satu VPS mati: VPS lain tetap melayani di nama host-nya sendiri; node peserta tetap tersambung lewat seed lain dan buku alamat |
| Pemindahan pertama (Q146, 17 November 2026) | Layanan VPS di PC dihentikan, volume `zvrg-data` (chain, `p2p_key` seed, indeks explorer), `zvrg-pengamat`, `zvrg-faucet`, `zvrg-faucet-dompet` disalin sebagai tar ke VPS pertama; PeerId seed tetap |
