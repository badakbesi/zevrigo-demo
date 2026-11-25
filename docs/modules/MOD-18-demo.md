# MOD-18: Layanan demo (`zevrigo-demo`)

- **ID:** `MOD-18`
- **Path:** akar repo `zevrigo-demo`; sampai DEC-39 `apps/demo` di repo inti
- **Paket:** `zevrigo-demo` (pustaka + bin)
- **Status:** 🟡 mode `rpc` (gerbang RPC, TSK-M53), alat `upacara` (TSK-M62), dan mode `faucet` (TSK-M54 bagian A); mode `daftar` belum (DEC-34, DEC-36)
- **Bergantung pada:** `MOD-01`, `MOD-02`, `MOD-08`, `MOD-09` (DEC-36; mode `rpc` hanya memakai `MOD-08`, `upacara` dan `faucet` memakai keempatnya)
- **Dipakai oleh:** —

## Tujuan

Menyediakan layanan publik jaringan demo regtest untuk pemula (Rancangan Ekosistem Demo v0.1): satu aplikasi dengan tiga mode
yang dijalankan sebagai tiga proses di belakang Cloudflare Tunnel, masing-masing dengan hostname sendiri.

## Tanggung Jawab

- Aplikasi desktop **Zevrigo Node** (DEC-44, RENCANA_T16) di `app/`: antarmuka React + backend Tauri 2 (`app/src-tauri`, workspace
  Cargo sendiri, bukan bagian program `zevrigo-demo`). Menjalankan `zevrigo-node --seed-resmi` dan `zevrigo-miner --blocks 1`
  dengan jeda 60/120/300 detik sebagai sidecar, hadiah ke alamat Zevrigo Wallet yang ditempel; tanpa kunci rahasia.
- Crate `zevrigo-node-mesin` di `mesin/` (DEC-45, Q196a): mesin node + penambang aplikasi Zevrigo Node (dipindah dari
  `app/src-tauri/src/mesin`), anggota workspace ini, dipakai aplikasi desktop (lewat path) dan mode `pengendali`.
- `zevrigo-demo pengendali --datadir <dir> --token-sha256 <berkas> [--http-bind <ip>] [--http-port <port>] [--rate <n>]
  [--trust-proxy-host <nama>] [--nyalakan-node]` (DEC-45, RENCANA_T17): pengendali jarak jauh untuk aplikasi Zevrigo Node di
  Android. Menjalankan node `--seed-resmi` dan penambang sendiri (program `zevrigo-node`, `zevrigo-miner` di samping program
  ini), tanpa Docker socket. Satu permintaan JSON-RPC 2.0 per `POST` dengan `Authorization: Bearer <token>`; hanya metode
  `info`, `keadaan_node`, `keadaan_tambang`, `log_terbaru`, `muat_pengaturan`, `simpan_pengaturan`, `periksa_alamat`,
  `nyalakan_node`, `hentikan_node`, `mulai_tambang`, `berhenti_tambang`. SHA-256 token di berkas (dibaca ulang setiap
  permintaan), perbandingan tanpa berhenti di byte pertama yang beda, `401` tanpa token yang benar, `429` untuk token salah
  berulang dan batas laju (bawaan 5 per detik), `405`, `413` di atas 16 KiB. Perintah kendali dicatat di log program dan log
  mesin. Port bawaan 28808. Sejak Q198 (DEC-46, jalur SSH; rancangan Cloudflare dibatalkan pencipta 25 November 2026):
  dicapai aplikasi Android lewat terowongan SSH ke 127.0.0.1; metode `saldo {alamat}` (UTXO alamat hadiah di node sendiri:
  matang/belum matang, ZVRG desimal); penanda `tambang-lanjut.json` di folder data (ditulis `mulai_tambang`, dihapus
  `berhenti_tambang`/`hentikan_node`) sehingga dengan `--nyalakan-node` penambang dilanjutkan sesudah mulai ulang begitu node
  sinkron; variabel `ZVRG_P2P_PUBLIK=/ip4|ip6/<ip>/tcp/<port>` menyalakan mode node publik di mesin (`--p2p-bind`,
  `--p2p-port`, `--p2p-publik`).
- Paket VPS `vps/` (Q198, nama dari pencipta): skrip bantu `demo-zevrigo-kendali` (perintah tetap: pasang, perbarui, token,
  status, node-publik, batas-cpu, amankan-ssh, cabut-kunci, restart-vps, copot), layanan systemd `demo-zevrigo-node` (pengguna
  `demo-zevrigo`, `/opt/demo-zevrigo/sekarang`, data `/var/lib/demo-zevrigo`, CPUQuota bawaan 50%, sandbox systemd), README,
  catatan rilis. Workflow `rilis-vps.yml` membangun `demo-zevrigo-vps-linux-x86_64.tar.gz` dan mengujinya sebagai layanan
  systemd sungguhan di Ubuntu 22.04; tag `demo-zevrigo-vps` = rilis draf.

- `zevrigo-demo rpc [--node-url <url>] [--http-bind <ip>] [--http-port <port>] [--rate <n>] [--trust-proxy]` (TSK-M53):
  gerbang RPC untuk wallet peserta di `127.0.0.1:28804` (bawaan). Satu permintaan JSON-RPC 2.0 per `POST`; hanya
  `get_chain_info`, `get_block_hash`, `get_block`, `get_mempool`, `scan_utxos`, `scan_mempool`, dan `send_raw_transaction`
  yang diteruskan ke node lewat `RpcClient` (MOD-08), dan hasil atau galat node (kode, pesan, `data`) dikembalikan apa adanya
  dengan `id` permintaan. Metode lain (termasuk `get_block_template` dan `submit_block`) `-32601`; batch `-32600`; badan di
  atas 256 KiB `413`; selain `POST` `405`; node tidak menjawab `-32603` tanpa rincian.
- Batas laju per pengunjung (bawaan 20 per detik, kapasitas 80) dan bersama (400 per detik), diperiksa sebelum badan dibaca;
  `429 {"error":"rate_limited"}` dengan `Retry-After: 1`. Modul `limit` adalah salinan pembatas MOD-14 (TSK-M40) karena MOD-18
  tidak bergantung pada MOD-14. Dengan `--trust-proxy`, alamat pengunjung dari `CF-Connecting-IP` hanya untuk koneksi dari
  loopback (pola TSK-M43). Alamat IP tidak disimpan dan tidak dicatat; log tanpa metode atau parameter.
- `zevrigo-demo upacara [--node-url <url>] --kantong <alamat>=<ZVRG>... [--fee-rate <n>] [--kirim]` (TSK-M62): alat operator
  sekali pakai untuk upacara genesis regtest. Memeriksa node (jaringan regtest, `network_id` = genesis regtest, output `G`
  kunci `1` di tinggi 0 indeks 0 masih ada, matang di blok berikutnya), lalu menyusun satu transaksi `G` → 1 sampai 16
  kantong alamat regtest jenis 0. Jumlah kantong wajib tepat 10.000.000 ZVRG; biaya (tarif × ukuran, bawaan 1 quanta per
  byte) diambil dari kantong terakhir. Tanda tangan BIP340 kunci `1` dengan `aux_rand` nol (kunci publik), diperiksa ulang
  sebelum dikirim. Tanpa `--kirim` hanya rencana dan txid. Bukan layanan dan tidak dibuka ke publik.
- `zevrigo-demo faucet --datadir <dir> [--node-url <url>] [--http-bind <ip>] [--http-port <port>] [--rate <n>] [--jumlah
  <ZVRG>] [--batas-harian <n>] [--fee-rate <n>] [--trust-proxy-host <nama>] [--allow-origin <origin>]...` (TSK-M54 bagian A):
  faucet demo di `127.0.0.1:28805` (bawaan). Wallet panas regtest: frasa 24 kata di `<datadir>/faucet-frasa.txt` (dibuat
  sekali, izin `0600`), alamat `m/44'/1'/0'/0/0` yang juga menjadi alamat kembalian; isi ulang manual dari kantong Faucet.
  `GET /api/v1/faucet`: alamat, jumlah per kiriman, saldo, sisa hari ini, siap/tidak. `POST /api/v1/faucet` badan
  `{"address": "rzvrg1..."}`: kirim `--jumlah` (bawaan 100 ZVRG) dengan pemilihan koin MOD-09. Batas (TSK-M72, Q182): sekali per alamat
  per 24 jam sejak kiriman terakhir ke alamat itu, paling banyak 5 kali per pengunjung per 24 jam (jendela bergeser), dan
  `--batas-harian` kiriman per hari UTC (bawaan 100); batas alamat dan harian disimpan di `<datadir>/faucet-batas.json` versi 2
  (hari, jumlah, alamat penerima dengan waktu kirim; versi 1 tetap dibaca), pengunjung hanya sebagai hash alamat IP dengan
  kunci acak di memori yang dibuang sesudah 24 jam. Galat batas pengunjung dan alamat memuat `retry_after` (detik) dan kepala
  `Retry-After`. Kiriman berantai lewat mempool (kembalian yang belum masuk blok langsung dipakai); setelah start, faucet
  menunggu sampai mempool bersih dari transaksi faucet (`busy`). Galat JSON `{"error", "message"}`: `400` alamat, `429` batas
  dan laju (bawaan 2 per detik), `503` kosong, sibuk, atau node, tanpa rincian node. CORS hanya untuk `--allow-origin`
  (bawaan explorer publik). `CF-Connecting-IP` dipercaya hanya dari alamat `--trust-proxy-host` (di-resolve ulang tiap 60
  detik). Log tanpa alamat IP dan tanpa alamat penerima.
- `zevrigo-demo web --web-dir <dir> [--http-bind <ip>] [--http-port <port>]` (TSK-M64, DEC-37): situs statis, dipakai
  untuk blog `../zevrigo-blog/out` (repo `zevrigo-blog`) di `127.0.0.1:28807` (bawaan). Modul `site` adalah salinan penyajian web MOD-14: jalur aman
  (paling banyak 8 segmen, tanpa `..` atau berkas tersembunyi, ekstensi dikenal ditambah `jpg` dan `webp`), `index.html` per
  folder, `404.html`, cache `immutable` untuk `_next/static`, kebijakan keamanan konten dengan hash SHA-256 skrip inline
  (`sha2`, paket yang sama dengan MOD-14), `nosniff`, tanpa referrer. Hanya `GET` dan `HEAD` (`405` untuk lainnya), tanpa
  kuki dan tanpa log pengunjung. Tidak mau start bila folder tanpa `index.html`.
- `daftar` (TSK-M55): belum tersedia (`mode daftar belum tersedia`).

## Bukan Tanggung Jawab

- Aturan konsensus atau validasi (`MOD-04`), penyimpanan chain (`MOD-05`), tampilan explorer (`MOD-14`).
- HTTPS: TLS diputus Cloudflare; gerbang hanya HTTP di loopback.
- Jaringan testnet dan mainnet. MOD-18 hanya untuk demo regtest.

## Task terkait
- [TSK-M53](../tasks/TSK-M53-gerbang-rpc.md) — 🟢 inisialisasi dan mode `rpc`
- [TSK-M62](../tasks/TSK-M62-alat-upacara-genesis.md) — 🟢 alat `upacara`
- [TSK-M54](../tasks/TSK-M54-faucet.md) — 🟡 bagian A: mode `faucet`; bagian B: compose, halaman explorer, anti-bot
- [TSK-M64](https://github.com/badakbesi/zevrigo-blog/blob/main/docs/tasks/TSK-M64-blog.md) — 🟢 mode `web` untuk blog
- [TSK-M55](https://github.com/badakbesi/zevrigo/blob/main/docs/tasks/README.md) — ⚪ mode `daftar`
