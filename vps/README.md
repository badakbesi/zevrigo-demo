# demo-zevrigo-vps: Zevrigo Node jaringan demo untuk VPS (Linux x86_64)

Paket ini dipasang oleh **aplikasi Zevrigo Node di Android** lewat SSH ke VPS Anda (RENCANA_T17, DEC-45, DEC-46). Anda tidak
perlu menjalankan apa pun sendiri: aplikasi mengunduh paket dari rilis resmi, memeriksa SHA-256, lalu memasangnya.
Koin jaringan demo tidak bernilai.

## Isi
- `zevrigo-node`, `zevrigo-miner`, `zevrigo-cli` (repo inti) dan `zevrigo-demo` (mode `pengendali`, repo ini)
- `demo-zevrigo-kendali`: skrip bantu dengan perintah tetap (pasang, perbarui, token, status, node-publik, batas-cpu,
  amankan-ssh, cabut-kunci, restart-vps, copot); lihat kepala skrip
- `demo-zevrigo-node.service`: layanan systemd

## Yang dipasang di VPS
| Tempat | Isi |
|---|---|
| pengguna sistem `demo-zevrigo` | menjalankan node (bukan root, tanpa shell) |
| `/opt/demo-zevrigo/sekarang` | program versi aktif (versi sebelumnya disimpan untuk kembali) |
| `/var/lib/demo-zevrigo` | chain, buku alamat rekan, pengaturan, SHA-256 token pengendali |
| `/etc/demo-zevrigo/node.env` | mode node publik (kosong = rekan biasa) |
| `demo-zevrigo-node.service` | layanan systemd, menyala sendiri sesudah VPS restart |
| `/usr/local/sbin/demo-zevrigo-kendali` | skrip bantu; `/etc/sudoers.d/demo-zevrigo` mengizinkan pengguna SSH menjalankannya tanpa password |

Tidak ada port baru yang dibuka: RPC node, P2P, dan pengendali hanya di 127.0.0.1 (aplikasi memakai terowongan SSH). Mode node
publik (pilihan) membuka port P2P. Tidak ada kunci wallet di VPS: hadiah tambang ke alamat Zevrigo Wallet Anda.

## Syarat
Linux x86_64, Ubuntu 22.04 atau lebih baru / Debian 12 atau lebih baru, systemd, akses root atau sudo, 1 vCPU, 1 GB RAM.

## Manual (tanpa aplikasi)
```bash
sha256sum demo-zevrigo-vps-linux-x86_64.tar.gz      # cocokkan dengan SHA256SUMS.txt rilis
tar xzf demo-zevrigo-vps-linux-x86_64.tar.gz
TOKEN=$(head -c 48 /dev/urandom | base64 | tr -d '/+=' | head -c 64); echo "simpan token ini: $TOKEN"
sudo ./demo-zevrigo-vps/demo-zevrigo-kendali pasang --token-sha256 "$(printf %s "$TOKEN" | sha256sum | cut -d' ' -f1)"
sudo demo-zevrigo-kendali status
```
Pengendali menjawab JSON-RPC di `http://127.0.0.1:28808` dengan `Authorization: Bearer <token>`.
