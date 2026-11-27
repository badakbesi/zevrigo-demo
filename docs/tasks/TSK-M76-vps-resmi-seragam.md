# TSK-M76 · Susunan VPS resmi seragam · 🟢 selesai · Prioritas: Tinggi
- **Jenis:** task repo (Docker) + VPS · **Modul:** — (`compose.publik.yaml`, `scripts/vps-resmi-*`, dokumen) · **Untuk:** FTR-11 (DEC-47)
- **Bergantung pada:** TSK-M65, TSK-M75 · **Ditulis:** Quick, 27 November 2026
- **Keputusan pencipta (27 November 2026, 14.36 sampai 14.47):** semua VPS resmi memakai satu susunan yang sama persis (tanpa
  versi per lokasi); isinya seed, node pengamat, explorer, dan semua layanan lain yang sekarang ada di VPS pertama (gerbang RPC,
  faucet, penambang cadangan, tunnel); hanya untuk VPS resmi kita, orang lain boleh memakai susunan berbeda; dikerjakan sekarang

## Isi
| Bagian | Isi |
|---|---|
| Compose (Q206a) | profile `vps` → `demo`; nama host seed dari `ZVRG_HOST_SEED`; seed memakai `--seed-resmi`; batas sumber daya seed dan explorer; tunnel lama ke profile `lama` (sementara); faucet menerima halaman explorer setiap VPS |
| Skrip (Q206a) | `scripts/vps-resmi.env.contoh`, `scripts/vps-resmi-pasang.sh` (salin compose dari commit, periksa, nyalakan, tulis `VERSI`, pulihkan bila gagal), `scripts/vps-resmi-periksa.sh` (bandingkan semua VPS) |
| VPS Eropa (Q206b) | dari `~/zevrigo-eu` (project `zevrigo-eu`) ke `~/zevrigo` (project `zevrigo-publik`): volume seed kedua dan pengamat kedua disalin ke nama baru, PeerId seed kedua tetap; explorer, gerbang RPC, faucet, penambang cadangan baru; route `demo-explorer-eu`, `demo-gerbang-rpc-eu`, `demo-faucet-eu` |
| VPS pertama (Q206c) | `.env` (`demo,lama`), compose baru, `VERSI`, image lama dibersihkan (aktif + satu sebelumnya) |
| Pemeriksaan (Q206d) | `scripts/vps-resmi-periksa.sh` = `SERAGAM`; versi Docker disamakan bila beda |

## Definition of done
- [x] Q206a commit compose, skrip, dan dokumen (c8fbba8, inti 15efb07 DEC-47; 27 November 2026)
- [x] Q206b VPS Eropa memakai susunan seragam (27 November 2026, 15.37): data seed kedua dan pengamat kedua disalin ke volume
  `zevrigo-publik_*` (sidik sama), PeerId seed kedua tetap `12D3KooWQEYr…CXTR`, 8 layanan jalan, explorer/RPC/faucet `-eu` menjawab
  dari internet; volume lama `zevrigo-eu_*` disimpan sampai Q206d
- [x] Q206c VPS pertama memakai susunan seragam (27 November 2026, 15.44): `.env` `demo,lama`, 9 layanan dibuat ulang dari compose
  seragam (`vps-resmi-pasang.sh` dengan `--force-recreate`, f3fb23b), PeerId seed pertama tetap, host baru dan lama menjawab, faucet
  menerima halaman explorer Eropa; image lama dibersihkan (aktif + `zevrigo:regtest-sebelumnya`)
- [x] Q206d `vps-resmi-periksa.sh zevrigo-vps zevrigo-eu` = `SERAGAM` (27 November 2026): paket Docker VPS demo disamakan dengan VPS
  Eropa, kedua VPS dipasang dari commit yang sama; sisa susunan lama VPS Eropa (`~/zevrigo-eu`, volume `zevrigo-eu_*`, token
  `~/.zevrigo-eu.env`) dihapus sesudah isi volume baru dicocokkan
