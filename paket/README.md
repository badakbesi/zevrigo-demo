# Paket peserta jaringan demo Zevrigo (regtest)

Paket ini berisi program untuk ikut menjalankan **jaringan demo Zevrigo**: node, miner, dan alat baris perintah (CLI) dengan
wallet. Jaringan demo memakai **regtest**: koin ZVRG di sini **tidak bernilai** dan chain bisa dimulai ulang. Tujuannya menguji
perangkat lunak dan membangun jaringan penguji sebelum testnet.

| Berkas | Isi |
|---|---|
| `zevrigo-node` | Node penuh: menyimpan chain, tersambung sendiri ke seed resmi jaringan demo (`--seed-resmi`) |
| `zevrigo-miner` | Penambang CPU: meminta template blok dari node lokal lalu menambang |
| `zevrigo-cli` | Wallet dan info chain lewat baris perintah (alamat, saldo, riwayat, kirim) |
| `zevrigo-demo` | Layanan jaringan demo (gerbang RPC, faucet, situs); tidak dibutuhkan untuk ikut menambang |
| `peserta-demo-zevrigo.sh` / `peserta-demo-zevrigo.cmd` | Skrip awal: node, wallet, lalu menambang satu blok setiap 2 menit |

Di Windows nama programnya berakhiran `.exe`. Unduh paket hanya dari halaman rilis resmi
https://github.com/badakbesi/zevrigo-demo/releases atau https://zevrigo.online/unduh/.

## 1. Periksa unduhan

Bandingkan SHA-256 berkas yang Anda unduh dengan `SHA256SUMS.txt` di halaman rilis. Harus sama persis.

```bash
sha256sum peserta-demo-zevrigo-*.tar.gz        # Linux
```

```powershell
Get-FileHash .\peserta-demo-zevrigo-*.zip -Algorithm SHA256   # Windows (PowerShell)
```

Program belum ditandatangani. Di Windows, klik kanan berkas `.zip`, pilih **Properties**, centang **Unblock**, lalu
**OK** sebelum diekstrak; kalau tidak, Windows bisa menahan setiap program saat dijalankan.

## 2. Mulai

**Linux** (x86_64, glibc 2.35 atau lebih baru, misalnya Ubuntu 22.04 atau Debian 12 ke atas):

```bash
tar xzf peserta-demo-zevrigo-linux-x86_64.tar.gz
cd peserta-demo-zevrigo
./peserta-demo-zevrigo.sh
```

**Windows** (10 atau 11, 64-bit): ekstrak `.zip`, buka folder `peserta-demo-zevrigo`, lalu klik dua kali
`peserta-demo-zevrigo.cmd` (atau jalankan dari Command Prompt di folder itu). Node menyala di jendela kedua; biarkan terbuka.

Yang dilakukan skrip awal:
1. Menyalakan node regtest yang tersambung sendiri ke seed resmi (`demo-seed-p2p.zevrigo.online`), lalu menunggu node
   mengejar chain jaringan demo. Data disimpan di `~/.zevrigo` (Linux) atau `%USERPROFILE%\.zevrigo` (Windows).
2. Bila belum ada wallet, membuat wallet baru: Anda membuat password (minimal 8 karakter) dan **mencatat 24 kata** yang
   tampil. Tanpa 24 kata itu wallet tidak bisa dipulihkan. Jangan memberikan 24 kata atau password kepada siapa pun.
3. Menambang **satu blok setiap 120 detik** (waktu blok target jaringan) dengan hadiah ke alamat baru wallet Anda. Jeda ini
   menjaga penambangan tetap adil bagi peserta lain dan tidak menghabiskan CPU.

Hentikan dengan **Ctrl-C**. Di Linux node ikut dimatikan; di Windows tutup juga jendela node dengan Ctrl-C.

Ganti jeda atau folder data lewat variabel lingkungan sebelum menjalankan skrip:

```bash
ZVRG_JEDA=300 ZVRG_DATA=/folder/lain ./peserta-demo-zevrigo.sh
```

```bat
set ZVRG_JEDA=300
set ZVRG_DATA=D:\zevrigo
peserta-demo-zevrigo.cmd
```

## 3. Perintah sendiri

```bash
./zevrigo-node --regtest --datadir ~/.zevrigo --seed-resmi      # node saja
./zevrigo-cli info                                              # tinggi chain node lokal
./zevrigo-cli --datadir ~/.zevrigo balance                      # saldo wallet
./zevrigo-cli --datadir ~/.zevrigo history                      # riwayat
./zevrigo-cli --datadir ~/.zevrigo address                      # alamat penerimaan baru
./zevrigo-cli --datadir ~/.zevrigo send --to <alamat> --amount <ZVRG>
./zevrigo-miner --payout <payload hex 64> --blocks 1            # satu blok, payload dari perintah address
```

Setiap program punya `--help`. Hadiah blok baru bisa dibelanjakan sesudah matang (100 blok). Koin awal untuk mencoba kirim
juga bisa diminta dari faucet: https://demo-explorer.zevrigo.online/faucet/.

## 4. Privasi dan keamanan

- Node tersambung ke seed resmi lewat Cloudflare; seed dan rekan lain melihat alamat IP Anda saat tersambung.
- RPC node hanya di `127.0.0.1` (tidak bisa dibuka dari komputer lain). Node tidak menerima sambungan masuk dari luar kecuali
  Anda menambah `--p2p-bind`; dengan opsi itu IP Anda terlihat oleh rekan dan port perlu diizinkan firewall.
- Node menyimpan alamat rekan yang ditemukan di `<folder data>/regtest/peers.json`, sehingga saat dibuka lagi tersambung
  tanpa menunggu seed.
- Wallet `<folder data>/regtest/wallet.json` dikunci password. Kehilangan file wallet bisa dipulihkan dengan 24 kata
  (`wallet-restore`).

## 5. Masalah umum

- **Windows: "VCRUNTIME140.dll tidak ditemukan".** Pasang Microsoft Visual C++ Redistributable x64 dari situs Microsoft
  (https://aka.ms/vs/17/release/vc_redist.x64.exe), lalu jalankan lagi.
- **"Node tidak menyala"**: port 28800 atau 28801 dipakai program lain (misalnya node lain). Hentikan node itu dulu.
- **Node tidak mengejar chain**: periksa internet. Seed resmi dan explorer: https://demo-explorer.zevrigo.online.

Laporan masalah: https://github.com/badakbesi/zevrigo-demo/issues. Kode sumber: https://github.com/badakbesi/zevrigo dan
https://github.com/badakbesi/zevrigo-demo. Lisensi: MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`).
