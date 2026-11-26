//! Layanan VPS aplikasi Zevrigo Node Android (Q199b): tahap aplikasi (buat kata sandi, sambung, pasang, amankan, terkunci,
//! siap), brankas di HP ([`super::brankas`]), dan satu sambungan SSH yang dipakai bersama untuk pengendali dan skrip bantu.
//! Dipanggil perintah Tauri (`perintah_vps`) dan uji `examples/uji-layanan.rs`. Rahasia tidak pernah keluar dari layanan ini
//! kecuali teks brankas terenkripsi ([`Layanan::ekspor`]).

use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use zeroize::Zeroizing;
use zevrigo_wallet_core::vault::KdfParams;

use super::brankas::{Rahasia, SANDI_MAKS, SANDI_MIN, buka_brankas, sandi_sah, tutup_brankas};
use super::kendali::{Kendali, StatusVps};
use super::pasang::{
    Kunci, PAKET_SHA256, PemeriksaanVps, ambil_alih, buat_token, pasang_baru, pasang_kunci_publik,
    periksa_vps, sha256_hex,
};
use super::sesi::{Galat, Keluaran, Masuk, Sasaran, Sesi};

/// Nama berkas brankas di folder data aplikasi.
pub const BERKAS_BRANKAS: &str = "vps-brankas.json";

/// Tahap aplikasi untuk antarmuka.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Tahap {
    /// Belum ada VPS di HP ini: buat kata sandi aplikasi, atau impor data dari HP lama.
    Baru,
    /// Kata sandi aplikasi sudah dibuat: isi alamat VPS, pengguna, dan password VPS.
    Sambung,
    /// Tersambung dengan password: hasil pemeriksaan VPS, lalu pasang.
    Pasang,
    /// Terpasang: matikan login password SSH.
    Amankan,
    /// Data VPS ada di HP, aplikasi terkunci.
    Terkunci,
    /// Siap dipakai.
    Siap,
}

/// Keadaan layanan untuk antarmuka (tanpa rahasia).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoLayanan {
    /// Tahap aplikasi.
    pub tahap: Tahap,
    /// Host VPS (tahap sambung sesudah dicoba, pasang, amankan, siap).
    pub host: Option<String>,
    /// Port SSH.
    pub port: Option<u16>,
    /// Pengguna SSH.
    pub pengguna: Option<String>,
    /// Sidik SHA-256 kunci host VPS.
    pub sidik: Option<String>,
    /// Hasil pemeriksaan VPS (tahap pasang).
    pub pemeriksaan: Option<PemeriksaanVps>,
    /// Alasan VPS tidak bisa dipasangi (tahap pasang; kosong = bisa).
    pub masalah: Vec<String>,
    /// Login password SSH sudah dimatikan aplikasi.
    pub amankan_selesai: bool,
    /// SHA-256 paket VPS yang dikenal aplikasi ini.
    pub paket_sha256: &'static str,
    /// Panjang kata sandi aplikasi paling sedikit.
    pub sandi_min: usize,
    /// Panjang kata sandi aplikasi paling banyak.
    pub sandi_maks: usize,
    /// Versi aplikasi.
    pub versi_aplikasi: &'static str,
}

/// Perintah skrip bantu dari antarmuka (bagian Pemeliharaan dan VPS). Perintah akses (token, amankan, cabut kunci, copot)
/// punya fungsi sendiri di [`Layanan`].
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "jenis", rename_all = "snake_case")]
pub enum PerintahUi {
    /// Perbarui ke paket VPS yang dikenal aplikasi.
    Perbarui,
    /// Batas CPU layanan (persen satu CPU).
    BatasCpu {
        /// 10 sampai 100.
        persen: u8,
    },
    /// Mode node publik.
    NodePublikNyala {
        /// IP publik VPS.
        ip: String,
        /// Port P2P.
        port: u16,
    },
    /// Node kembali menjadi rekan biasa.
    NodePublikMati,
    /// Restart VPS.
    RestartVps,
}

impl PerintahUi {
    /// Perintah skrip bantu.
    ///
    /// # Errors
    ///
    /// [`Galat::Masukan`] bila IP tidak sah.
    pub fn ke_kendali(&self) -> Result<Kendali, Galat> {
        Ok(match self {
            Self::Perbarui => Kendali::Perbarui,
            Self::BatasCpu { persen } => Kendali::BatasCpu(*persen),
            Self::NodePublikNyala { ip, port } => Kendali::NodePublikNyala {
                ip: ip
                    .trim()
                    .parse::<IpAddr>()
                    .map_err(|_| Galat::Masukan("IP publik VPS tidak sah".to_owned()))?,
                port: *port,
            },
            Self::NodePublikMati => Kendali::NodePublikMati,
            Self::RestartVps => Kendali::RestartVps,
        })
    }
}

/// Pemasangan pertama yang sedang berjalan.
struct Awal {
    sandi: Zeroizing<String>,
    sasaran: Option<Sasaran>,
    sandi_vps: Option<Zeroizing<String>>,
    sidik: Option<String>,
    pemeriksaan: Option<PemeriksaanVps>,
}

/// Brankas yang sudah dibuka.
struct Terbuka {
    rahasia: Rahasia,
    sandi: Zeroizing<String>,
    lewati_amankan: bool,
}

#[derive(Default)]
struct Keadaan {
    awal: Option<Awal>,
    terbuka: Option<Terbuka>,
}

/// Layanan VPS: satu per aplikasi (satu VPS per aplikasi, RENCANA_T17 bagian 8).
pub struct Layanan {
    berkas: PathBuf,
    kdf: KdfParams,
    keadaan: Mutex<Keadaan>,
    sesi: tokio::sync::Mutex<Option<Arc<Sesi>>>,
}

impl std::fmt::Debug for Layanan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layanan")
            .field("berkas", &self.berkas)
            .finish_non_exhaustive()
    }
}

/// Menulis berkas brankas: berkas sementara lalu ganti nama (tidak setengah tertulis), izin 600 di Linux dan Android.
fn tulis_berkas(berkas: &Path, teks: &str) -> Result<(), Galat> {
    let gagal = |e: std::io::Error| Galat::Brankas(format!("tidak bisa ditulis: {e}"));
    if let Some(folder) = berkas.parent() {
        std::fs::create_dir_all(folder).map_err(gagal)?;
    }
    let sementara = berkas.with_extension("json.baru");
    tulis_privat(&sementara, teks).map_err(gagal)?;
    std::fs::rename(&sementara, berkas).map_err(gagal)
}

#[cfg(unix)]
fn tulis_privat(berkas: &Path, teks: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(berkas)?;
    f.write_all(teks.as_bytes())?;
    f.sync_all()
}

#[cfg(not(unix))]
fn tulis_privat(berkas: &Path, teks: &str) -> std::io::Result<()> {
    std::fs::write(berkas, teks)
}

impl Layanan {
    /// Layanan dengan brankas di `berkas` dan parameter Argon2id bawaan wallet (64 MiB, 3 putaran).
    #[must_use]
    pub fn baru(berkas: PathBuf) -> Self {
        Self::dengan_kdf(berkas, KdfParams::DEFAULT)
    }

    /// Layanan dengan parameter Argon2id lain (uji). Brankas yang sudah ada dibuka dengan parameter di berkasnya.
    #[must_use]
    pub fn dengan_kdf(berkas: PathBuf, kdf: KdfParams) -> Self {
        Self {
            berkas,
            kdf,
            keadaan: Mutex::new(Keadaan::default()),
            sesi: tokio::sync::Mutex::new(None),
        }
    }

    fn kunci_keadaan(&self) -> Result<MutexGuard<'_, Keadaan>, Galat> {
        self.keadaan
            .lock()
            .map_err(|_| Galat::Sistem("keadaan aplikasi rusak".to_owned()))
    }

    fn ada_brankas(&self) -> bool {
        self.berkas.is_file()
    }

    /// Keadaan untuk antarmuka.
    ///
    /// # Errors
    ///
    /// [`Galat::Sistem`] bila keadaan aplikasi rusak.
    pub fn info(&self) -> Result<InfoLayanan, Galat> {
        let ada_brankas = self.ada_brankas();
        let k = self.kunci_keadaan()?;
        let mut info = InfoLayanan {
            tahap: Tahap::Baru,
            host: None,
            port: None,
            pengguna: None,
            sidik: None,
            pemeriksaan: None,
            masalah: Vec::new(),
            amankan_selesai: false,
            paket_sha256: PAKET_SHA256,
            sandi_min: SANDI_MIN,
            sandi_maks: SANDI_MAKS,
            versi_aplikasi: env!("CARGO_PKG_VERSION"),
        };
        if let Some(t) = &k.terbuka {
            info.tahap = if t.rahasia.amankan_selesai || t.lewati_amankan {
                Tahap::Siap
            } else {
                Tahap::Amankan
            };
            info.host = Some(t.rahasia.host.clone());
            info.port = Some(t.rahasia.port);
            info.pengguna = Some(t.rahasia.pengguna.clone());
            info.sidik = Some(t.rahasia.sidik.clone());
            info.amankan_selesai = t.rahasia.amankan_selesai;
        } else if let Some(a) = &k.awal {
            info.tahap = if a.pemeriksaan.is_some() {
                Tahap::Pasang
            } else {
                Tahap::Sambung
            };
            if let Some(s) = &a.sasaran {
                info.host = Some(s.host().to_owned());
                info.port = Some(s.port());
                info.pengguna = Some(s.pengguna().to_owned());
            }
            info.sidik.clone_from(&a.sidik);
            info.masalah = a
                .pemeriksaan
                .as_ref()
                .map(PemeriksaanVps::masalah)
                .unwrap_or_default();
            info.pemeriksaan.clone_from(&a.pemeriksaan);
        } else if ada_brankas {
            info.tahap = Tahap::Terkunci;
        }
        Ok(info)
    }

    /// Kata sandi aplikasi baru (tahap baru). Brankas baru ditulis sesudah pemasangan berhasil.
    ///
    /// # Errors
    ///
    /// [`Galat::Tahap`] bila data VPS sudah ada di HP, [`Galat::Masukan`] bila sandi tidak memenuhi syarat.
    pub fn buat_sandi(&self, sandi: &str) -> Result<InfoLayanan, Galat> {
        let ada_brankas = self.ada_brankas();
        {
            let mut k = self.kunci_keadaan()?;
            if k.terbuka.is_some() || ada_brankas {
                return Err(Galat::Tahap("data VPS sudah ada di HP ini"));
            }
            if !sandi_sah(sandi) {
                return Err(Galat::Masukan(format!(
                    "kata sandi aplikasi {SANDI_MIN} sampai {SANDI_MAKS} karakter"
                )));
            }
            k.awal = Some(Awal {
                sandi: Zeroizing::new(sandi.to_owned()),
                sasaran: None,
                sandi_vps: None,
                sidik: None,
                pemeriksaan: None,
            });
        }
        self.info()
    }

    async fn lepas_sesi(&self) {
        let lama = self.sesi.lock().await.take();
        if let Some(sesi) = lama {
            sesi.tutup().await;
        }
    }

    /// Membatalkan pemasangan pertama (kembali ke tahap baru; password VPS dilupakan).
    ///
    /// # Errors
    ///
    /// [`Galat::Sistem`] bila keadaan aplikasi rusak.
    pub async fn batal(&self) -> Result<InfoLayanan, Galat> {
        let awal = self.kunci_keadaan()?.awal.take();
        drop(awal);
        self.lepas_sesi().await;
        self.info()
    }

    /// Sambungan pertama dengan password VPS: sidik kunci host dipercaya, VPS diperiksa (tanpa mengubah apa pun).
    ///
    /// # Errors
    ///
    /// [`Galat::Tahap`], [`Galat::Masukan`], [`Galat::MasukDitolak`], galat sambungan.
    pub async fn sambung_awal(
        &self,
        host: &str,
        port: u16,
        pengguna: &str,
        sandi_vps: &str,
    ) -> Result<InfoLayanan, Galat> {
        let sasaran = Sasaran::baru(host, port, pengguna)?;
        if sandi_vps.is_empty() {
            return Err(Galat::Masukan("password VPS kosong".to_owned()));
        }
        let ada_awal = self.kunci_keadaan()?.awal.is_some();
        if !ada_awal {
            return Err(Galat::Tahap("buat kata sandi aplikasi dulu"));
        }
        self.lepas_sesi().await;
        let sandi = Zeroizing::new(sandi_vps.to_owned());
        let sesi = Sesi::sambung(&sasaran, &Masuk::Sandi(sandi.clone()), None).await?;
        let pemeriksaan = match periksa_vps(&sesi).await {
            Ok(p) => p,
            Err(e) => {
                sesi.putus().await;
                return Err(e);
            }
        };
        let sidik = sesi.sidik().to_owned();
        let simpan = {
            let mut k = self.kunci_keadaan()?;
            if let Some(awal) = k.awal.as_mut() {
                awal.sasaran = Some(sasaran);
                awal.sandi_vps = Some(sandi);
                awal.sidik = Some(sidik);
                awal.pemeriksaan = Some(pemeriksaan);
                true
            } else {
                false
            }
        };
        if !simpan {
            sesi.putus().await;
            return Err(Galat::Tahap("pemasangan dibatalkan"));
        }
        *self.sesi.lock().await = Some(Arc::new(sesi));
        self.info()
    }

    /// Pemasangan (tahap pasang): paket VPS resmi dipasang (atau node yang sudah ada diambil alih dengan token baru), kunci
    /// SSH aplikasi dipasang, sambungan baru dengan kunci dan sidik yang sama dibuktikan, lalu brankas ditulis.
    ///
    /// # Errors
    ///
    /// [`Galat::Tahap`], [`Galat::Masukan`] (VPS tidak memenuhi syarat, password sudo salah), [`Galat::Perintah`],
    /// [`Galat::Brankas`], galat sambungan.
    pub async fn pasang(&self) -> Result<InfoLayanan, Galat> {
        let (sasaran, sandi_vps, sandi, sidik, terpasang) = {
            let k = self.kunci_keadaan()?;
            let a = k.awal.as_ref().ok_or(Galat::Tahap("sambung ke VPS dulu"))?;
            let (Some(sasaran), Some(sandi_vps), Some(sidik), Some(p)) =
                (&a.sasaran, &a.sandi_vps, &a.sidik, &a.pemeriksaan)
            else {
                return Err(Galat::Tahap("sambung ke VPS dulu"));
            };
            let masalah = p.masalah();
            if !masalah.is_empty() {
                return Err(Galat::Masukan(masalah.join("; ")));
            }
            (
                sasaran.clone(),
                sandi_vps.clone(),
                a.sandi.clone(),
                sidik.clone(),
                p.terpasang,
            )
        };
        let sesi = self
            .sesi
            .lock()
            .await
            .clone()
            .ok_or(Galat::Tahap("sambungan VPS terputus; sambung lagi"))?;
        let token = buat_token()?;
        let kunci = Kunci::baru()?;
        let token_sha = sha256_hex(&token);
        if terpasang {
            ambil_alih(&sesi, Some(sandi_vps.as_str()), &token_sha).await?;
        } else {
            pasang_baru(&sesi, Some(sandi_vps.as_str()), &token_sha).await?;
        }
        pasang_kunci_publik(&sesi, &kunci).await?;
        // Bukti: sambungan baru dengan kunci aplikasi dan sidik yang sama; skrip bantu dan pengendali menjawab.
        let baru = Sesi::sambung(&sasaran, &kunci.masuk(), Some(&sidik)).await?;
        let status = baru.status_vps().await?;
        if status.layanan != "active" {
            baru.tutup().await;
            return Err(Galat::Perintah {
                kode: None,
                pesan: format!("layanan Zevrigo Node {}", status.layanan),
            });
        }
        if let Err(e) = baru.rpc(&token, "info", json!({})).await {
            baru.tutup().await;
            return Err(e);
        }
        let rahasia = Rahasia {
            host: sasaran.host().to_owned(),
            port: sasaran.port(),
            pengguna: sasaran.pengguna().to_owned(),
            sidik,
            kunci: kunci.ke_openssh()?.as_str().to_owned(),
            token: token.as_str().to_owned(),
            amankan_selesai: false,
        };
        self.simpan(&rahasia, &sandi)?;
        {
            let mut k = self.kunci_keadaan()?;
            k.awal = None;
            k.terbuka = Some(Terbuka {
                rahasia,
                sandi,
                lewati_amankan: false,
            });
        }
        let lama = self.sesi.lock().await.replace(Arc::new(baru));
        if let Some(lama) = lama {
            lama.tutup().await;
        }
        self.info()
    }

    fn simpan(&self, rahasia: &Rahasia, sandi: &str) -> Result<(), Galat> {
        let teks = tutup_brankas(rahasia, sandi, self.kdf)?;
        tulis_berkas(&self.berkas, &teks)
    }

    /// Mematikan login password SSH VPS (tahap amankan), lalu brankas diperbarui.
    ///
    /// # Errors
    ///
    /// [`Galat::Terkunci`], [`Galat::Perintah`] (misalnya pengguna SSH belum punya kunci), galat sambungan.
    pub async fn amankan(&self) -> Result<InfoLayanan, Galat> {
        self.kendali(&Kendali::AmankanSsh).await?;
        let (rahasia, sandi) = {
            let mut k = self.kunci_keadaan()?;
            let t = k.terbuka.as_mut().ok_or(Galat::Terkunci)?;
            t.rahasia.amankan_selesai = true;
            (t.rahasia.clone(), t.sandi.clone())
        };
        self.simpan(&rahasia, &sandi)?;
        self.info()
    }

    /// Melewati tahap amankan untuk sesi ini (ditanyakan lagi saat aplikasi dibuka berikutnya).
    ///
    /// # Errors
    ///
    /// [`Galat::Terkunci`].
    pub fn lewati_amankan(&self) -> Result<InfoLayanan, Galat> {
        {
            let mut k = self.kunci_keadaan()?;
            k.terbuka.as_mut().ok_or(Galat::Terkunci)?.lewati_amankan = true;
        }
        self.info()
    }

    /// Membuka brankas dengan kata sandi aplikasi. Sambungan ke VPS dibuat saat dibutuhkan.
    ///
    /// # Errors
    ///
    /// [`Galat::SandiSalah`], [`Galat::Brankas`].
    pub fn buka(&self, sandi: &str) -> Result<InfoLayanan, Galat> {
        let teks = std::fs::read_to_string(&self.berkas)
            .map(Zeroizing::new)
            .map_err(|e| Galat::Brankas(format!("tidak terbaca: {e}")))?;
        let rahasia = buka_brankas(&teks, sandi)?;
        {
            let mut k = self.kunci_keadaan()?;
            k.awal = None;
            k.terbuka = Some(Terbuka {
                rahasia,
                sandi: Zeroizing::new(sandi.to_owned()),
                lewati_amankan: false,
            });
        }
        self.info()
    }

    /// Mengunci aplikasi: rahasia dihapus dari memori, sambungan ditutup (node di VPS tetap berjalan).
    ///
    /// # Errors
    ///
    /// [`Galat::Sistem`] bila keadaan aplikasi rusak.
    pub async fn kunci(&self) -> Result<InfoLayanan, Galat> {
        let lama = {
            let mut k = self.kunci_keadaan()?;
            (k.terbuka.take(), k.awal.take())
        };
        drop(lama);
        self.lepas_sesi().await;
        self.info()
    }

    fn rahasia_terbuka<T>(&self, ambil: impl FnOnce(&Rahasia) -> T) -> Result<T, Galat> {
        let k = self.kunci_keadaan()?;
        k.terbuka
            .as_ref()
            .map(|t| ambil(&t.rahasia))
            .ok_or(Galat::Terkunci)
    }

    /// Sambungan dengan kunci aplikasi (dibuat bila belum ada).
    async fn sesi_kunci(&self) -> Result<Arc<Sesi>, Galat> {
        let (sasaran, kunci, sidik) = self.rahasia_terbuka(|r| {
            (
                Sasaran::baru(&r.host, r.port, &r.pengguna),
                Kunci::dari_openssh(&r.kunci),
                r.sidik.clone(),
            )
        })?;
        let mut slot = self.sesi.lock().await;
        if let Some(sesi) = slot.as_ref() {
            return Ok(Arc::clone(sesi));
        }
        let sesi = Arc::new(Sesi::sambung(&sasaran?, &kunci?.masuk(), Some(&sidik)).await?);
        *slot = Some(Arc::clone(&sesi));
        Ok(sesi)
    }

    async fn buang_sesi(&self, lama: &Arc<Sesi>) {
        let mut slot = self.sesi.lock().await;
        if slot.as_ref().is_some_and(|s| Arc::ptr_eq(s, lama)) {
            *slot = None;
        }
    }

    /// Memanggil metode pengendali dengan token aplikasi. Sambungan yang putus dibuat ulang sekali.
    ///
    /// # Errors
    ///
    /// [`Galat::Terkunci`], [`Galat::Masukan`], [`Galat::TokenDitolak`], [`Galat::Pengendali`], [`Galat::SidikBerubah`],
    /// galat sambungan.
    pub async fn rpc(&self, metode: &str, params: Value) -> Result<Value, Galat> {
        let token = self.rahasia_terbuka(|r| Zeroizing::new(r.token.clone()))?;
        let sesi = self.sesi_kunci().await?;
        match sesi.rpc(&token, metode, params.clone()).await {
            Err(e) if e.putus() => {
                self.buang_sesi(&sesi).await;
                if matches!(e, Galat::Sambung(_)) {
                    let sesi = self.sesi_kunci().await?;
                    sesi.rpc(&token, metode, params).await
                } else {
                    Err(e)
                }
            }
            hasil => hasil,
        }
    }

    /// Menjalankan perintah skrip bantu. Sambungan yang putus sebelum perintah terkirim dibuat ulang sekali.
    ///
    /// # Errors
    ///
    /// [`Galat::Terkunci`], [`Galat::Masukan`], [`Galat::Perintah`], galat sambungan.
    pub async fn kendali(&self, perintah: &Kendali) -> Result<Keluaran, Galat> {
        let sesi = self.sesi_kunci().await?;
        match sesi.kendali(perintah).await {
            Err(e) if e.putus() => {
                self.buang_sesi(&sesi).await;
                if matches!(e, Galat::Sambung(_)) {
                    let sesi = self.sesi_kunci().await?;
                    sesi.kendali(perintah).await
                } else {
                    Err(e)
                }
            }
            hasil => hasil,
        }
    }

    /// Keadaan VPS dari skrip bantu.
    ///
    /// # Errors
    ///
    /// Seperti [`Layanan::kendali`], atau [`Galat::Perintah`] bila JSON tidak terbaca.
    pub async fn status(&self) -> Result<StatusVps, Galat> {
        let keluaran = self.kendali(&Kendali::Status).await?;
        serde_json::from_slice(keluaran.stdout.trim_ascii()).map_err(|e| Galat::Perintah {
            kode: Some(0),
            pesan: format!("status tidak terbaca: {e}"),
        })
    }

    /// Mengganti token pengendali dengan token baru, lalu brankas diperbarui.
    ///
    /// # Errors
    ///
    /// Seperti [`Layanan::kendali`], atau [`Galat::Brankas`].
    pub async fn ganti_token(&self) -> Result<(), Galat> {
        let baru = buat_token()?;
        self.kendali(&Kendali::Token(sha256_hex(&baru))).await?;
        let (rahasia, sandi) = {
            let mut k = self.kunci_keadaan()?;
            let t = k.terbuka.as_mut().ok_or(Galat::Terkunci)?;
            baru.as_str().clone_into(&mut t.rahasia.token);
            (t.rahasia.clone(), t.sandi.clone())
        };
        self.simpan(&rahasia, &sandi)
    }

    /// Teks brankas terenkripsi untuk pindah HP (dibuka di HP baru dengan kata sandi aplikasi yang sama).
    ///
    /// # Errors
    ///
    /// [`Galat::Terkunci`], [`Galat::Brankas`].
    pub fn ekspor(&self) -> Result<String, Galat> {
        self.rahasia_terbuka(|_| ())?;
        std::fs::read_to_string(&self.berkas)
            .map_err(|e| Galat::Brankas(format!("tidak terbaca: {e}")))
    }

    /// Data VPS dari HP lama (teks [`Layanan::ekspor`]) dengan kata sandi aplikasi HP lama (tahap baru).
    ///
    /// # Errors
    ///
    /// [`Galat::Tahap`] bila data VPS sudah ada, [`Galat::SandiSalah`], [`Galat::Brankas`].
    pub fn impor(&self, teks: &str, sandi: &str) -> Result<InfoLayanan, Galat> {
        let ada = self.ada_brankas() || self.kunci_keadaan()?.terbuka.is_some();
        if ada {
            return Err(Galat::Tahap("data VPS sudah ada di HP ini"));
        }
        let rahasia = buka_brankas(teks, sandi)?;
        tulis_berkas(&self.berkas, teks.trim())?;
        {
            let mut k = self.kunci_keadaan()?;
            k.awal = None;
            k.terbuka = Some(Terbuka {
                rahasia,
                sandi: Zeroizing::new(sandi.to_owned()),
                lewati_amankan: false,
            });
        }
        self.info()
    }

    /// Menghapus data VPS dari HP ini (node di VPS tidak disentuh). Juga untuk kata sandi aplikasi yang lupa.
    ///
    /// # Errors
    ///
    /// [`Galat::Brankas`] bila berkas tidak bisa dihapus.
    pub async fn lupakan(&self) -> Result<InfoLayanan, Galat> {
        self.kunci().await?;
        match std::fs::remove_file(&self.berkas) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(Galat::Brankas(format!("tidak bisa dihapus: {e}"))),
        }
        self.info()
    }

    /// Mencabut akses HP ini: kunci SSH aplikasi dihapus dari authorized_keys VPS, lalu data VPS dihapus dari HP.
    ///
    /// # Errors
    ///
    /// Seperti [`Layanan::kendali`] dan [`Layanan::lupakan`].
    pub async fn cabut_akses(&self) -> Result<InfoLayanan, Galat> {
        let blob = self
            .rahasia_terbuka(|r| Kunci::dari_openssh(&r.kunci))??
            .blob_publik()?;
        self.kendali(&Kendali::CabutKunci(blob)).await?;
        self.lupakan().await
    }

    /// Mencopot Zevrigo Node dari VPS (data chain disimpan kecuali `hapus_data`), lalu data VPS dihapus dari HP.
    ///
    /// # Errors
    ///
    /// Seperti [`Layanan::kendali`] dan [`Layanan::lupakan`].
    pub async fn copot(&self, hapus_data: bool) -> Result<InfoLayanan, Galat> {
        self.kendali(&Kendali::Copot { hapus_data }).await?;
        self.lupakan().await
    }
}
