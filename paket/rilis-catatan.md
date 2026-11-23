Paket peserta **jaringan demo Zevrigo (regtest)**: node, miner, CLI (wallet), dan layanan demo untuk Linux dan Windows, dengan
skrip awal yang menyalakan node tersambung ke seed resmi (`--seed-resmi`), membuat wallet, lalu menambang satu blok setiap 120
detik. Koin jaringan demo tidak bernilai. Panduan lengkap ada di `README.md` di dalam paket.

Dibangun GitHub Actions dari tag `{{TAG}}` (commit `{{COMMIT}}` repo zevrigo-demo) dan repo inti `badakbesi/zevrigo` commit
`{{INTI}}`, dengan `cargo build --release --locked`.

| Berkas | Sistem | SHA-256 |
|---|---|---|
| `{{BERKAS_LINUX}}` | Linux x86_64 (glibc 2.35 atau lebih baru) | `{{SHA_LINUX}}` |
| `{{BERKAS_WINDOWS}}` | Windows 10 dan 11, x64 | `{{SHA_WINDOWS}}` |

Periksa: `sha256sum {{BERKAS_LINUX}}` (Linux) atau `Get-FileHash .\{{BERKAS_WINDOWS}} -Algorithm SHA256` (PowerShell). Hasilnya harus
sama dengan baris di atas dan dengan `SHA256SUMS.txt`.

- **Linux:** `tar xzf {{BERKAS_LINUX}}`, `cd peserta-demo-zevrigo`, `./peserta-demo-zevrigo.sh`.
- **Windows:** klik kanan `.zip` > Properties > centang Unblock > OK, ekstrak, lalu klik dua kali `peserta-demo-zevrigo.cmd`.
  Bila muncul "VCRUNTIME140.dll tidak ditemukan", pasang Microsoft Visual C++ Redistributable x64
  (https://aka.ms/vs/17/release/vc_redist.x64.exe).
- **Belum ditandatangani** (DEC-40: rilis node bertanda tangan ditunda). Jalankan hanya jika SHA-256 cocok.
- Node hanya membuka RPC di 127.0.0.1 dan tidak menerima sambungan masuk dari luar secara bawaan. Seed dan rekan melihat alamat IP
  Anda saat tersambung.
