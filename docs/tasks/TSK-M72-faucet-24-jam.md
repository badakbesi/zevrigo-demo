# TSK-M72 · Faucet: batas 24 jam per alamat dan 5 per pengunjung · 🟢 Q182 · Prioritas: Tinggi
- **Jenis:** task modul · **Modul:** MOD-18 (`zevrigo-demo faucet`), MOD-14 (antarmuka web explorer) · **Untuk:** FTR-11
- **Bergantung pada:** TSK-M54 · **Ditulis:** Quick, 22 November 2026 (pencipta 22 November 2026: usul Quick, digabung dengan
  perbaikan tampilan HP explorer)

## Masalah
Sebelumnya: sekali per pengunjung (hash alamat IP) dan sekali per alamat per hari UTC. Wallet kedua dari HP, PC, atau jaringan
yang sama (banyak operator seluler berbagi satu alamat IP) langsung ditolak `visitor_limit`, sehingga faucet terasa "hanya bisa
sekali". Faucet tidak mengenal wallet (hanya alamat), jadi "per wallet" dijalankan sebagai per alamat.

## Isi
| Bagian | Isi |
|---|---|
| Batas alamat | Sekali per alamat per 24 jam sejak kiriman terakhir ke alamat itu (`CLAIM_WINDOW_SECONDS` 86 400), bukan sampai 00.00 UTC |
| Batas pengunjung | Paling banyak `VISITOR_LIMIT` = 5 kiriman per pengunjung dalam 24 jam terakhir (jendela bergeser). Hash alamat IP dengan kunci acak yang dibuat saat start, hanya di memori, dibuang sesudah 24 jam |
| Batas harian | Tetap: `--batas-harian` kiriman per hari UTC untuk semua pengunjung (bawaan 100) |
| File batas | `faucet-batas.json` versi 2: `hari`, `jumlah`, `alamat` = objek alamat → waktu kirim terakhir (detik Unix), yang lebih tua dari 24 jam dibuang. Versi 1 tetap dibaca: alamat hari itu dicatat sejak 00.00 UTC hari itu (boleh lagi mulai 00.00 UTC berikutnya, seperti aturan lama) |
| HTTP | `GET` menambah `visitor_limit` dan `claim_window_seconds`. Galat `visitor_limit` dan `address_limit` (429) menambah `retry_after` (detik) dan kepala `Retry-After`; pesan `coba lagi … lagi` (menit dibulatkan ke atas) |
| Explorer | Teks halaman faucet (Indonesia, Inggris): aturan 24 jam, 5 per pengunjung, catatan privasi 24 jam |
| Dependensi | Tanpa paket baru; Cargo.lock dan package-lock.json tidak berubah |
| Test | `faucet` +2: jendela 24 jam (2 per pengunjung, alamat 24 jam sejak kirim, `retry_after`), file versi 1; test lama disesuaikan (versi 2, `retry_after`, `wait_text`) |

## Pasang
Q182: image `zevrigo:regtest` baru, di VPS hanya container `explorer` dan `faucet` yang dibuat ulang (node, saldo, frasa, dan
volume faucet tetap). Image lama diberi tag `zevrigo:regtest-q181` di PC dan VPS untuk kembali.
