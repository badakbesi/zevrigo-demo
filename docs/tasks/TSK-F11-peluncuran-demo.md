# TSK-F11 · Demo publik regtest untuk pemula · 🟡 bagian 1 selesai · Prioritas: Tinggi
- **Jenis:** task fitur (FTR-11, DEC-34) · **Modul:** repo (Docker), MOD-18, MOD-12
- **Bergantung pada:** TSK-M47, TSK-M53, TSK-M54 bagian A, TSK-M62, TSK-M63 · **Memblokir:** TSK-M59, tahap 2
- **Ditulis:** Quick, 13 November 2026

## Bagian 1: peluncuran tertutup (Q133)
Urutan Rancangan Ekosistem Demo v0.1 Bagian 4.3, dijalankan satu skrip:

| Langkah | Isi |
|---|---|
| Sebelum mati | Daftar kantong `ai_advisory_hub/treasury_kantong.txt` diperiksa (8 alamat, urutan Bagian 4.2, sha256 tetap). Seed hanya tersambung ke 4 node operator. Image `zevrigo:regtest` dibangun ulang (alat `upacara`, mode `faucet`, `send --dari`); image lama disimpan sebagai `zevrigo:sebelum-q133` |
| Mati dan cadangan | `docker compose down` (volume tetap). Kelima volume (`zvrg-data`, `zvrg-penambang`, `zvrg-treasury`, `zvrg-faucet`, `zvrg-pengamat`) dicadangkan sebagai tar di `~/zevrigo-cadangan/q133-<waktu>/` (di luar repo, izin 700) |
| Reset | Dihapus: `regtest/chain.redb`, `regtest/explorer.sqlite*`, `regtest/history.json`. Tetap: `regtest/p2p_key` (PeerId sama), `regtest/wallet.json`, `payout-penambang` |
| Tertutup | Hanya node seed, tanpa tunnel. 99 blok ke alamat penambang (pilihan pencipta). Rencana upacara diperiksa terhadap daftar kantong, lalu `upacara --kirim`, 1 blok. `scan_utxos` di seed: 8 kantong tepat satu UTXO, jumlah persis, kantong Faucet dikurangi biaya; `G` habis; 100 coinbase penambang |
| Buka | Explorer, gerbang RPC, `cloudflared`; 4 node operator sinkron; saldo wallet treasury = 10.000.000 ZVRG − biaya, 8 UTXO; layanan `faucet` (saldo 0 sampai isi ulang pertama); `miner-berkala` lanjut dari blok 101; dari internet gerbang RPC menunjukkan 8 kantong yang sama |
| Sesudah skrip (pencipta) | Route `demo-faucet.zevrigo.online` → `http://node-faucet:28805`; isi ulang faucet pertama 10.000 ZVRG dengan `zevrigo-cli send --dari <kantong Faucet>` di node treasury (butuh password treasury) |

Batas yang diumumkan tetap berlaku (Bagian 4.3): node mana pun dengan chain lebih berat yang tersambung kemudian bisa
menimpa chain ini. Chain uji lama (tinggi ±870) hanya ada di cadangan.

## Bagian 2 (tahap 2)
Pendaftaran Investor Demo, distribusi vesting dari kantong Investor Demo, pengumuman (Rancangan Bagian 6, tahap 2).

## Mulai ulang kedua (19 November 2026, RENCANA_REGTEST R3)
Keputusan pencipta 19 November 2026: data chain lama di PC dan VPS dihapus tanpa arsip, wallet penambang, treasury, dan faucet
baru, PeerId seed baru (`12D3KooWABPBsu8gZWtL7bbeEPjg6gwRPDbp3GZidpTeUC95rRDo`), image dari repo `zevrigo-demo`. Peluncuran tertutup dengan pola bagian 1 (99 blok,
upacara, blok 100) di PC, lalu volume seed dipindah ke VPS (pola TSK-M65). Risiko "chain demo bisa ditulis ulang siapa pun"
diterima dan diumumkan di blog dan `SUMBER_RESMI.md` repo inti. Langkah: Q156 inventaris, Q157/Q157b hapus PC dan seed baru,
q157_wallet.sh wallet baru, Q158 commit dan peluncuran tertutup, Q159 VPS.

## Mulai ulang ketiga (28 November 2026, Q211 sampai Q212)
Keputusan pencipta 27 November 2026 22.20 dan 22.28: jaringan demo dari blok 0 sesudah pembersihan total (Q211): semua container,
volume, image, kunci seed, wallet operator, wallet faucet, dan token tunnel di PC dan VPS dihapus tanpa cadangan. Image baru, seed
baru PeerId `12D3KooWPdd2VH92LVRwJNrokxKQJCerctJNksErJn5PB53kbEsD` (Q212a), wallet penambang dan treasury baru dengan delapan kantong
bernama sama (Q212b), seed bawaan node diganti (Q212c), peluncuran tertutup pola bagian 1 (99 blok, upacara, blok 100; Q212d), lalu
VPS demo dan isi faucet pertama 50.000 ZVRG (Q212e). Satu VPS resmi (DEC-48). Rilis node dengan seed baru (paket VPS, Zevrigo Node
desktop dan Android) menyusul.
