# Zevrigo Demo (MOD-18)

Layanan jaringan demo regtest Zevrigo Coin (DEC-34, DEC-36): `zevrigo-demo rpc` (gerbang RPC aman untuk wallet peserta,
`https://demo-gerbang-rpc.zevrigo.online`), `faucet` (100 ZVRG demo per permintaan, sekali per alamat
dan paling banyak 5 kali per pengunjung dalam 24 jam), `upacara` (alat upacara genesis
regtest), dan `web` (penyaji situs statis untuk blog). Koin demo tidak bernilai.

Repo ini bagian dari **zevrigo_lab**: crate inti dari repo inti `zevrigo` di folder sebelah (`../zevrigo`). Aturan proyek:
[AGENTS.md](AGENTS.md). Repo ini juga memegang paket Docker regtest dan jaringan demo publik di VPS dan PC (TSK-M33,
TSK-M65, DEC-38).

```bash
make check-all
cargo run -- rpc --help
```

Rancangan: [docs/reference/rancangan-ekosistem-demo-v0.1.md](docs/reference/rancangan-ekosistem-demo-v0.1.md). Modul:
[docs/modules/MOD-18-demo.md](docs/modules/MOD-18-demo.md).

## Docker (regtest)

Tanpa memasang Rust atau Node.js: butuh Docker Engine dengan plugin compose. Image dibangun dari folder induk
`zevrigo_lab`, jadi repo `zevrigo`, `zevrigo-explorer`, dan `zevrigo-blog` harus ada di sebelah repo ini. Dari folder
`zevrigo-demo`:

```bash
docker compose build
docker compose up -d
```

Buka `http://127.0.0.1:28802/`. Node dan explorer berjalan di container dengan data di volume `zevrigo_zvrg-data`;
RPC node tidak terbuka ke host, dan explorer hanya bisa dibuka dari komputer ini. Wallet dan miner di dalam container:

```bash
docker compose exec node zevrigo-cli --datadir /data wallet-create
docker compose exec node zevrigo-cli --datadir /data address
ZVRG_PAYOUT=<payload dari perintah address> docker compose --profile miner up -d miner
docker compose down
```

Aturan lengkap di [docs/reference/spesifikasi-docker-v0.1.md](docs/reference/spesifikasi-docker-v0.1.md).

### Explorer dan seed publik lewat Cloudflare Tunnel

`compose.publik.yaml` membuka explorer, seed P2P, gerbang RPC, dan faucet ke internet lewat Cloudflare Tunnel tanpa membuka
port router. Ada dua tunnel (DEC-42): `zevrigo` (layanan `cloudflared-zevrigo`, domain resmi `zevrigo.online`) dan tunnel
lama (layanan `cloudflared`, `ratuaurion.store`, tetap jalan selama peralihan karena wallet 0.3.1 dan aplikasi Explorer 0.1.0
memakainya). Token disimpan di luar repo, di VPS: `~/.zevrigo.env` dan `~/.zevrigo-tunnel.env` (izin `600`, satu baris
`TUNNEL_TOKEN=` diikuti token dari dasbor Cloudflare). Route kedua tunnel tipe HTTP ke layanan yang sama: explorer
`demo-explorer.zevrigo.online` (lama `zevrigo.ratuaurion.store`) ke `localhost:28802`, seed P2P WebSocket
`demo-seed-p2p.zevrigo.online` (lama `zevrigo-seed`) ke `localhost:28803`, gerbang RPC untuk wallet peserta (hanya metode aman,
tanpa menambang) `demo-gerbang-rpc.zevrigo.online` (lama `zevrigo-rpc`) ke `localhost:28804`, dan faucet
`demo-faucet.zevrigo.online` (lama `zevrigo-faucet`) ke `node-faucet:28805`.

Jaringan demo berjalan di dua mesin (TSK-M65, DEC-38), dipilih dengan profile compose. Tanpa `COMPOSE_PROFILES` tidak ada
layanan publik yang dinyalakan, sehingga seed dan tunnel tidak pernah hidup di dua tempat sekaligus:

```bash
# PC (WSL): membangun image, lalu node penambang + miner-berkala dan node treasury
docker compose -f compose.yaml -f compose.publik.yaml build node
COMPOSE_PROFILES=pc docker compose -f compose.yaml -f compose.publik.yaml up -d
# VPS (image dari PC lewat docker save/load): seed, explorer, gerbang RPC, blog, tunnel, faucet, node pengamat
COMPOSE_PROFILES=vps docker compose -f compose.yaml -f compose.publik.yaml up -d --no-build
```

RPC dan P2P TCP node tidak terbuka, explorer dan seed hanya bisa dicapai lewat tunnel, dan batas laju explorer berlaku per
pengunjung (`--trust-proxy`). Selain seed, empat node operator jaringan demo (`node-penambang`, `node-treasury`,
`node-faucet`, `node-pengamat`) berjalan di container sendiri dan tersambung ke seed lewat alamat publik, sama seperti
peserta. Layanan `miner-berkala` di node penambang menambang satu blok regtest setiap 120 detik (waktu blok target),
sehingga explorer publik terus menampilkan blok baru. Jeda bisa diganti dengan `ZVRG_JEDA=<detik>` saat `up`, dan layanan ini
bisa dihentikan sendiri dengan `COMPOSE_PROFILES=pc docker compose -f compose.yaml -f compose.publik.yaml stop miner-berkala`. Rancangan jaringan
demo: [docs/reference/rancangan-ekosistem-demo-v0.1.md](docs/reference/rancangan-ekosistem-demo-v0.1.md). Node lain bergabung
ke seed dengan seed resmi bawaan node (DEC-43), tanpa menyalin alamat:

```bash
zevrigo-node --regtest --datadir ~/.zevrigo --seed-resmi
```

atau dengan alamat lengkap `--connect /dns4/demo-seed-p2p.zevrigo.online/tcp/443/tls/sni/demo-seed-p2p.zevrigo.online/ws/p2p/<PeerId seed>`.
Seed mengumumkan alamat itu ke rekan lewat pertukaran alamat, dan node menyimpan rekan yang ditemukan di
`<folder>/regtest/peers.json`.

Saat ini regtest (uji); testnet menyusul. Rincian di Spesifikasi Docker v0.1 Bagian 7.

## Paket peserta (rilis `peserta-demo-zevrigo`)

Untuk peserta tanpa Docker (RENCANA_T14 bagian 5b): arsip Linux x86_64 (`.tar.gz`) dan Windows x64 (`.zip`) berisi
`zevrigo-node`, `zevrigo-miner`, `zevrigo-cli`, `zevrigo-demo`, README, dan skrip awal `peserta-demo-zevrigo.sh` /
`peserta-demo-zevrigo.cmd`. Skrip menyalakan node dengan `--seed-resmi`, membuat wallet bila belum ada, lalu menambang satu blok
setiap 120 detik (`ZVRG_JEDA`). Dibangun workflow [rilis-peserta.yml](.github/workflows/rilis-peserta.yml) saat tag
`peserta-demo-zevrigo` di-push (repo inti dikunci `ZEVRIGO_CORE_REF`), sebagai rilis draf yang diterbitkan pencipta. Isi README
paket dan skrip ada di folder [paket/](paket/).

Lisensi: MIT OR Apache-2.0 (DEC-02).

## Aplikasi Zevrigo Node (`app/`)

Aplikasi desktop Windows dan Linux (DEC-44): node dan penambang jaringan demo dengan antarmuka, tanpa terminal. Sidecar
`zevrigo-node` dan `zevrigo-miner` sama dengan paket peserta; hadiah ke alamat Zevrigo Wallet yang ditempel. Rincian di
[app/README.md](app/README.md); dibangun dan diuji workflow [rilis-node-app.yml](.github/workflows/rilis-node-app.yml).

## Pengendali jarak jauh (`pengendali`) dan mesin (`mesin/`)

Mode `zevrigo-demo pengendali` (DEC-45) menjalankan node dan penambang di VPS dan menerima perintah bertoken dari aplikasi
Zevrigo Node di Android (RENCANA_T17). Mesinnya crate [`mesin/`](mesin), sama dengan aplikasi desktop. Rincian opsi di
`zevrigo-demo --help` dan [MOD-18](docs/modules/MOD-18-demo.md).

Paket VPS di [`vps/`](vps) (DEC-46): aplikasi Android memasangnya lewat SSH sebagai layanan systemd `demo-zevrigo-node` dengan
skrip bantu `demo-zevrigo-kendali`, tanpa port baru; dibangun dan diuji workflow
[rilis-vps.yml](.github/workflows/rilis-vps.yml), tag `demo-zevrigo-vps`.
