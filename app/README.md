# Zevrigo Node (aplikasi)

Aplikasi desktop untuk peserta jaringan demo Zevrigo (regtest): menyalakan node (`--seed-resmi`), menampilkan status sinkron,
dan menambang satu blok setiap 1, 2, atau 5 menit dengan hadiah ke alamat Zevrigo Wallet yang ditempel. Aplikasi tidak membuat
dan tidak menyimpan kunci rahasia. RENCANA_T16, DEC-44.

- Antarmuka: React di `src/` (dari Google AI Studio, pencipta 23 November 2026). Di browser (`npm run dev`) memakai backend
  tiruan `src/lib/mockBackend.ts`; di aplikasi memakai perintah Tauri (`src/lib/backend.ts`).
- Backend: `src-tauri/` (Tauri 2, workspace Cargo sendiri). `zevrigo-node` dan `zevrigo-miner` dari repo inti dibawa sebagai
  sidecar (`src-tauri/binaries/<nama>-<target triple>`), sama dengan paket peserta `peserta-demo-zevrigo`.
- Folder data sama dengan paket peserta: `~/.zevrigo` (Linux), `%USERPROFILE%\.zevrigo` (Windows); ganti dengan `ZVRG_DATA`.

- Android (RENCANA_T17 bagian 13, DEC-46, mulai Q199a): fitur Cargo `vps` (mati di desktop) menambah modul `src-tauri/src/vps`:
  SSH ke VPS pengguna (russh + ring), sidik kunci host dipercaya pertama kali lalu dikunci, pemasangan paket VPS resmi
  `demo-zevrigo-vps` (SHA-256 tertanam di aplikasi), skrip bantu `demo-zevrigo-kendali` dengan perintah tetap, dan JSON-RPC
  pengendali lewat terowongan SSH. Uji sungguhan ke VPS: `cargo run --features vps --example uji-vps` (masukan `ZVRG_UJI_*`).
- Mode VPS di antarmuka (Q199b): build dengan fitur `vps` menjawab `mode_aplikasi` = `vps`, lalu antarmuka memakai
  `src/lib/vps.ts` (perintah `vps_*` di `src-tauri/src/perintah_vps.rs`) dan layar `src/components/vps` (kata sandi aplikasi,
  sambung, pasang, amankan, terkunci, lalu kartu node/menambang/log ditambah saldo, VPS, pemeliharaan, akses). Kata sandi
  aplikasi paling sedikit 8 karakter; brankas `vps-brankas.json` di folder data aplikasi (Argon2id + XChaCha20-Poly1305,
  vault MOD-09). Di browser: `npm run dev`, buka `http://127.0.0.1:1420/?mode=vps` (tiruan, tanpa VPS). Uji layanan
  sungguhan: `cargo run --features vps --example uji-layanan`.

```bash
npm ci --ignore-scripts && npm run check      # antarmuka
make app-check                                 # dari akar repo: fmt, clippy, larangan, test Rust (butuh sidecar)
```
