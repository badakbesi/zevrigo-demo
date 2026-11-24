# Zevrigo Node (aplikasi)

Aplikasi desktop untuk peserta jaringan demo Zevrigo (regtest): menyalakan node (`--seed-resmi`), menampilkan status sinkron,
dan menambang satu blok setiap 1, 2, atau 5 menit dengan hadiah ke alamat Zevrigo Wallet yang ditempel. Aplikasi tidak membuat
dan tidak menyimpan kunci rahasia. RENCANA_T16, DEC-44.

- Antarmuka: React di `src/` (dari Google AI Studio, pencipta 23 November 2026). Di browser (`npm run dev`) memakai backend
  tiruan `src/lib/mockBackend.ts`; di aplikasi memakai perintah Tauri (`src/lib/backend.ts`).
- Backend: `src-tauri/` (Tauri 2, workspace Cargo sendiri). `zevrigo-node` dan `zevrigo-miner` dari repo inti dibawa sebagai
  sidecar (`src-tauri/binaries/<nama>-<target triple>`), sama dengan paket peserta `peserta-demo-zevrigo`.
- Folder data sama dengan paket peserta: `~/.zevrigo` (Linux), `%USERPROFILE%\.zevrigo` (Windows); ganti dengan `ZVRG_DATA`.

```bash
npm ci --ignore-scripts && npm run check      # antarmuka
make app-check                                 # dari akar repo: fmt, clippy, larangan, test Rust (butuh sidecar)
```
