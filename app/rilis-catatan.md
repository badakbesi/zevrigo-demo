Aplikasi **Zevrigo Node {{VERSI}}** untuk **jaringan demo Zevrigo (regtest)**, Windows dan Linux: menyalakan node yang
tersambung sendiri ke seed resmi (`--seed-resmi`), menampilkan status sinkron, dan menambang satu blok setiap 1, 2, atau 5 menit
dengan hadiah ke alamat Zevrigo Wallet yang Anda tempel. Aplikasi tidak membuat dan tidak menyimpan kunci rahasia. Koin jaringan
demo tidak bernilai.

Dibangun GitHub Actions dari tag `{{TAG}}` (commit `{{COMMIT}}` repo zevrigo-demo, folder `app/`) dan repo inti
`badakbesi/zevrigo` commit `{{INTI}}` (node dan miner, sama dengan paket peserta `peserta-demo-zevrigo`), dengan `--locked`.

| Berkas | Sistem | SHA-256 |
|---|---|---|
| `{{BERKAS_WINDOWS}}` | Windows 10 dan 11, x64 (installer) | `{{SHA_WINDOWS}}` |
| `{{BERKAS_LINUX}}` | Linux x86_64 (AppImage) | `{{SHA_LINUX}}` |

Periksa: `Get-FileHash .\{{BERKAS_WINDOWS}} -Algorithm SHA256` (PowerShell) atau `sha256sum {{BERKAS_LINUX}}` (Linux). Hasilnya harus
sama dengan baris di atas dan dengan `SHA256SUMS.txt`.

- **Windows:** jalankan installer. Bila muncul "Windows melindungi PC Anda": Info selengkapnya > Tetap jalankan (hanya jika SHA-256
  cocok). Lalu buka **Zevrigo Node** dari menu Start.
- **Linux:** `chmod +x {{BERKAS_LINUX}}` lalu `./{{BERKAS_LINUX}}`. Butuh FUSE 2 (Ubuntu/Debian: `sudo apt install libfuse2`,
  Ubuntu 24.04: `libfuse2t64`).
- **Cara pakai:** klik Nyalakan node, tunggu status Sinkron, tempel alamat penerimaan dari Zevrigo Wallet (`rzvrg1...`), pilih jeda
  (2 menit disarankan), klik Mulai menambang. Hadiah blok bisa dipakai sesudah 100 blok. Menutup jendela menghentikan penambang lalu
  node dengan rapi.
- Folder data sama dengan paket peserta: `%USERPROFILE%\.zevrigo` (Windows), `~/.zevrigo` (Linux). Jangan jalankan aplikasi dan
  skrip paket peserta bersamaan.
- **Belum ditandatangani** (DEC-40). Node hanya membuka RPC di 127.0.0.1 dan tidak menerima sambungan masuk dari luar. Seed dan rekan
  melihat alamat IP Anda saat tersambung. Kebijakan privasi: `PRIVASI-DAN-LISENSI.txt` di installer.
