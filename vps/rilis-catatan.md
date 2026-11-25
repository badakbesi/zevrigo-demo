Paket **Zevrigo Node untuk VPS** (jaringan demo regtest, Linux x86_64): node, penambang, CLI, pengendali jarak jauh, skrip bantu
`demo-zevrigo-kendali`, dan layanan systemd. Dipasang dan dikendalikan aplikasi Zevrigo Node di Android lewat SSH. Koin jaringan
demo tidak bernilai.

Dibangun GitHub Actions dari tag `{{TAG}}` (commit `{{COMMIT}}` repo zevrigo-demo) dan repo inti `badakbesi/zevrigo` commit
`{{INTI}}`, dengan `cargo build --release --locked`, lalu dipasang dan diuji sebagai layanan systemd di Ubuntu 22.04.

| Berkas | Sistem | SHA-256 |
|---|---|---|
| `{{BERKAS}}` | Linux x86_64, Ubuntu 22.04+ / Debian 12+ | `{{SHA}}` |

- Belum ditandatangani (DEC-40). Pasang hanya bila SHA-256 cocok; aplikasi memeriksanya sendiri.
- Tanpa port baru (RPC, P2P, pengendali hanya 127.0.0.1; node publik pilihan). Tanpa kunci wallet di VPS.
- Isi dan cara pasang manual: `README.md` di dalam paket.
