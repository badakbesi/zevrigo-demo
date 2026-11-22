# Zevrigo Demo (MOD-18)

Layanan jaringan demo regtest Zevrigo Coin (DEC-34, DEC-36): `zevrigo-demo rpc` (gerbang RPC aman untuk wallet peserta,
`https://zevrigo-rpc.ratuaurion.store`), `faucet` (100 ZVRG demo per permintaan, sekali per alamat
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

`compose.publik.yaml` membuka explorer dan seed P2P ke internet lewat Cloudflare Tunnel tanpa membuka port router. Token
tunnel disimpan di luar repo, di `~/.zevrigo-tunnel.env` (izin `600`, satu baris `TUNNEL_TOKEN=` diikuti token dari dasbor
Cloudflare Zero Trust). Public hostname tunnel, semuanya tipe HTTP: `zevrigo.ratuaurion.store` ke `localhost:28802`
(explorer), `zevrigo-seed.ratuaurion.store` ke `localhost:28803` (seed P2P WebSocket), dan `zevrigo-rpc.ratuaurion.store` ke
`localhost:28804` (gerbang RPC untuk wallet peserta: hanya metode aman, tanpa menambang).

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
ke seed dengan:

```bash
zevrigo-node --regtest --datadir ~/.zevrigo \
  --connect /dns4/zevrigo-seed.ratuaurion.store/tcp/443/tls/sni/zevrigo-seed.ratuaurion.store/ws/p2p/<PeerId seed>
```

Saat ini regtest (uji); testnet menyusul. Rincian di Spesifikasi Docker v0.1 Bagian 7.

Lisensi: MIT OR Apache-2.0 (DEC-02).
