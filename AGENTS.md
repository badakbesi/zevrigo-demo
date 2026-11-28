# AGENTS.md: Zevrigo Demo (MOD-18)

Layanan jaringan demo regtest: gerbang RPC aman (`rpc`), faucet (`faucet`), alat upacara genesis (`upacara`), penyaji situs
statis (`web`), dan pendaftaran (`daftar`, belum). Crate `zevrigo-demo` di akar repo. Bergantung pada MOD-01, 02, 08, 09
dari repo inti (DEC-36). Hanya regtest.

## Hubungan dengan repo inti
Repo ini bagian dari **zevrigo_lab** (pemisahan repo 17 November 2026, AGENDA P1). Repo inti `zevrigo` di folder sebelah
(`../zevrigo`) adalah sumber kebenaran: parameter protokol, aturan konsensus, larangan mutlak, pembagian peran (DEC-16),
keputusan (`docs/decisions`), peta modul, dan penomoran DEC/MOD/TSK. **Baca `../zevrigo/AGENTS.md` dulu; semua aturannya
berlaku di sini.** Bila aturan di file ini dan di repo inti bertentangan, ikuti repo inti dan laporkan.

## Larangan mutlak (ringkas, rincian di repo inti)
- Tanpa `unsafe` dan tanpa floating point di kode proyek, termasuk test. `make check-forbidden` memeriksanya.
- Jumlah koin selalu `u128` quanta di Rust; di TypeScript `bigint` atau `string`, tidak pernah `number`.
- Aturan konsensus hanya di crate `zevrigo-consensus` repo inti. Repo ini tidak menulis aturan konsensus.
- Menambah atau mengubah versi dependensi wajib minta izin pencipta.

## Git
- Conventional Commits. Jangan `git push`, jangan force, jangan mengubah riwayat, remote, identitas, atau `git config`.
  Push dilakukan pencipta (DEC-32). Jangan memakai GitHub CLI atau akun online tanpa izin tertulis.
- Status sekarang: **belum repo git** (keputusan git menunggu pencipta).

## Perintah
```bash
make check-all   # fmt-check + lint + check-forbidden + test
make docker-build # image zevrigo:regtest, konteks folder induk zevrigo_lab (Dockerfile.dockerignore)
```

## Khusus repo ini
- Paket Docker (`Dockerfile`, `Dockerfile.dockerignore`, `compose.yaml`, `compose.publik.yaml`) dan operasi VPS (DEC-38) ada
  di repo ini sejak 18 November 2026. Konteks build adalah folder induk `zevrigo_lab`, jadi repo `zevrigo`,
  `zevrigo-explorer`, dan `zevrigo-blog` harus berdampingan; yang masuk konteks hanya daftar izin di
  `Dockerfile.dockerignore`. Image dibangun di PC lalu dikirim ke VPS (docker save/load).
- Nama project compose (`zevrigo-regtest-peserta` di `compose.yaml`, `zevrigo-regtest` bila digabung `compose.publik.yaml`) dan nama
  volume jangan diganti tanpa langkah pindah data: data node VPS dan PC ada di volume itu (pindah pertama Q214, 28 November 2026).
- Pola nama di mesin operator: berkas jaringan demo regtest hanya di `~/zevrigo-regtest/` (VPS: `susunan/`, `rahasia/`), berkas
  rilis hanya di `~/zevrigo-rilis/`; tidak ada nama Zevrigo lain di luar keduanya, repo, dan `~/.ssh/zevrigo-*`.
- Token tunnel, wallet faucet, dan kunci treasury tidak pernah masuk repo.

## Berhenti dan tanya bila
- Ada parameter protokol yang tampak salah, atau tugas butuh dependensi baru, `sudo`, atau perubahan di luar repo ini.
- Perlu mengubah crate di repo inti `zevrigo` (itu tugas terpisah di repo inti).
