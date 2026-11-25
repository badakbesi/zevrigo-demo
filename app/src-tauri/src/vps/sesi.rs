//! Sambungan SSH ke VPS: masuk dengan password atau kunci aplikasi, sidik kunci host, menjalankan perintah tetap.

use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use russh::keys::{HashAlg, PrivateKey, PrivateKeyWithHashAlg, PublicKeyOrCertificate};
use russh::{ChannelMsg, Disconnect, client};
use zeroize::Zeroizing;

/// Batas waktu tersambung dan masuk ke VPS.
const BATAS_SAMBUNG: Duration = Duration::from_secs(30);
/// Batas keluaran satu perintah (stdout dan stderr masing-masing).
const BATAS_KELUARAN: usize = 1 << 20;

/// Alamat VPS dan nama pengguna SSH.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sasaran {
    host: String,
    port: u16,
    pengguna: String,
}

impl Sasaran {
    /// Memeriksa masukan pengguna: host (IP atau nama: huruf, angka, titik, titik dua, tanda hubung), port, nama pengguna Linux.
    ///
    /// # Errors
    ///
    /// [`Galat::Masukan`] bila salah satu tidak sah.
    pub fn baru(host: &str, port: u16, pengguna: &str) -> Result<Self, Galat> {
        let host = host.trim();
        let host_sah = !host.is_empty()
            && host.len() <= 253
            && host
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | ':' | '-'));
        if !host_sah {
            return Err(Galat::Masukan("alamat VPS tidak sah".to_owned()));
        }
        if port == 0 {
            return Err(Galat::Masukan("port SSH tidak sah".to_owned()));
        }
        if !nama_pengguna_sah(pengguna) {
            return Err(Galat::Masukan("nama pengguna SSH tidak sah".to_owned()));
        }
        Ok(Self {
            host: host.to_owned(),
            port,
            pengguna: pengguna.to_owned(),
        })
    }

    /// Host VPS.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Port SSH.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Nama pengguna SSH.
    #[must_use]
    pub fn pengguna(&self) -> &str {
        &self.pengguna
    }
}

/// Nama pengguna Linux yang juga diterima skrip bantu (`^[a-z_][a-z0-9_-]{0,31}$`).
pub(super) fn nama_pengguna_sah(nama: &str) -> bool {
    let mut huruf = nama.chars();
    nama.len() <= 32
        && matches!(huruf.next(), Some('a'..='z' | '_'))
        && huruf.all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | '-'))
}

/// Cara masuk ke VPS.
pub enum Masuk {
    /// Password dari penyedia VPS: hanya untuk pemasangan pertama, lalu dilupakan aplikasi.
    Sandi(Zeroizing<String>),
    /// Kunci ed25519 aplikasi (dipasang di authorized_keys saat pemasangan).
    Kunci(Arc<PrivateKey>),
}

impl fmt::Debug for Masuk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Sandi(_) => "Masuk::Sandi(..)",
            Self::Kunci(_) => "Masuk::Kunci(..)",
        })
    }
}

/// Galat sambungan VPS, dengan pesan untuk pengguna.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Galat {
    /// Masukan pengguna atau argumen perintah tidak sah.
    Masukan(String),
    /// Galat di HP (misalnya sumber acak sistem).
    Sistem(String),
    /// VPS tidak bisa dihubungi atau sambungan SSH putus.
    Sambung(String),
    /// Sidik kunci host berbeda dengan yang disimpan aplikasi (kemungkinan penyadapan, atau VPS dipasang ulang).
    SidikBerubah {
        /// Sidik yang disimpan aplikasi.
        dikenal: String,
        /// Sidik yang dikirim VPS sekarang.
        terlihat: String,
    },
    /// Nama pengguna, password, atau kunci ditolak VPS.
    MasukDitolak,
    /// Perintah di VPS gagal.
    Perintah {
        /// Kode keluar (tidak ada bila perintah tidak selesai).
        kode: Option<u32>,
        /// Baris galat dari VPS.
        pesan: String,
    },
    /// Batas waktu habis.
    BatasWaktu(&'static str),
    /// Pengendali tidak menjawab dengan benar.
    Pengendali(String),
    /// Token pengendali ditolak (HTTP 401).
    TokenDitolak,
}

impl fmt::Display for Galat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Masukan(pesan) => f.write_str(pesan),
            Self::Sistem(pesan) => write!(f, "galat di HP: {pesan}"),
            Self::Sambung(pesan) => write!(f, "tidak tersambung ke VPS: {pesan}"),
            Self::SidikBerubah { dikenal, terlihat } => write!(
                f,
                "sidik kunci VPS berubah (disimpan {dikenal}, sekarang {terlihat}); sambungan ditolak"
            ),
            Self::MasukDitolak => f.write_str("VPS menolak nama pengguna, password, atau kunci"),
            Self::Perintah {
                kode: Some(kode),
                pesan,
            } => write!(f, "perintah di VPS gagal (kode {kode}): {pesan}"),
            Self::Perintah { kode: None, pesan } => {
                write!(f, "perintah di VPS tidak selesai: {pesan}")
            }
            Self::BatasWaktu(apa) => write!(f, "batas waktu habis: {apa}"),
            Self::Pengendali(pesan) => write!(f, "pengendali di VPS: {pesan}"),
            Self::TokenDitolak => f.write_str("token pengendali ditolak VPS"),
        }
    }
}

impl std::error::Error for Galat {}

/// Galat pustaka SSH atau I/O menjadi [`Galat::Sambung`].
pub(super) fn sambung(e: impl fmt::Display) -> Galat {
    Galat::Sambung(e.to_string())
}

/// Keluaran satu perintah di VPS.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Keluaran {
    /// Kode keluar (tidak ada bila VPS tidak mengirimnya).
    pub kode: Option<u32>,
    /// Keluaran standar (dipotong pada 1 MiB).
    pub stdout: Vec<u8>,
    /// Keluaran galat (dipotong pada 1 MiB).
    pub stderr: Vec<u8>,
}

impl Keluaran {
    /// `true` bila kode keluar 0.
    #[must_use]
    pub fn berhasil(&self) -> bool {
        self.kode == Some(0)
    }

    /// Keluaran standar sebagai teks.
    #[must_use]
    pub fn teks(&self) -> String {
        String::from_utf8_lossy(&self.stdout).trim_end().to_owned()
    }

    /// Baris galat terakhir skrip bantu (`galat: ...`), atau baris terakhir keluaran galat.
    #[must_use]
    pub fn pesan_galat(&self) -> String {
        let teks = String::from_utf8_lossy(&self.stderr);
        let baris: Vec<&str> = teks
            .lines()
            .map(str::trim)
            .filter(|b| !b.is_empty())
            .collect();
        baris
            .iter()
            .rev()
            .find_map(|b| b.strip_prefix("galat: "))
            .or_else(|| baris.last().copied())
            .unwrap_or("tanpa pesan")
            .to_owned()
    }

    /// Keluaran ini bila berhasil.
    ///
    /// # Errors
    ///
    /// [`Galat::Perintah`] bila kode keluar bukan 0.
    pub fn wajib_berhasil(self) -> Result<Self, Galat> {
        if self.berhasil() {
            Ok(self)
        } else {
            Err(Galat::Perintah {
                kode: self.kode,
                pesan: self.pesan_galat(),
            })
        }
    }
}

fn tambah(tujuan: &mut Vec<u8>, data: &[u8]) {
    let sisa = BATAS_KELUARAN.saturating_sub(tujuan.len());
    tujuan.extend_from_slice(data.get(..sisa.min(data.len())).unwrap_or(data));
}

/// Pemeriksa kunci host: mencatat sidik SHA-256 kunci host VPS dan menolaknya bila berbeda dengan sidik yang disimpan.
pub(super) struct Penjaga {
    dikenal: Option<String>,
    terlihat: Arc<Mutex<Option<String>>>,
}

impl client::Handler for Penjaga {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        kunci: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        // Sertifikat host tidak dipakai VPS biasa: ditolak.
        let PublicKeyOrCertificate::PublicKey { key, .. } = kunci else {
            return Ok(false);
        };
        let sidik = key.fingerprint(HashAlg::Sha256).to_string();
        let cocok = self.dikenal.as_deref().is_none_or(|d| d == sidik);
        if let Ok(mut terlihat) = self.terlihat.lock() {
            *terlihat = Some(sidik);
        }
        Ok(cocok)
    }
}

/// Sambungan SSH yang sudah masuk ke VPS.
pub struct Sesi {
    pub(super) handle: client::Handle<Penjaga>,
    sidik: String,
    root: bool,
}

impl fmt::Debug for Sesi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sesi")
            .field("sidik", &self.sidik)
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl Sesi {
    /// Tersambung dan masuk. `sidik_dikenal`: sidik kunci host yang disimpan aplikasi; tidak ada pada sambungan pertama (sidik
    /// VPS dipercaya, lalu [`Sesi::sidik`] disimpan aplikasi).
    ///
    /// # Errors
    ///
    /// [`Galat::SidikBerubah`], [`Galat::MasukDitolak`], [`Galat::Sambung`], [`Galat::BatasWaktu`].
    pub async fn sambung(
        sasaran: &Sasaran,
        masuk: &Masuk,
        sidik_dikenal: Option<&str>,
    ) -> Result<Self, Galat> {
        let terlihat = Arc::new(Mutex::new(None));
        let penjaga = Penjaga {
            dikenal: sidik_dikenal.map(str::to_owned),
            terlihat: Arc::clone(&terlihat),
        };
        let konfigurasi = Arc::new(client::Config {
            inactivity_timeout: Some(Duration::from_secs(600)),
            keepalive_interval: Some(Duration::from_secs(15)),
            nodelay: true,
            ..client::Config::default()
        });
        let hasil = tokio::time::timeout(
            BATAS_SAMBUNG,
            client::connect(konfigurasi, (sasaran.host.as_str(), sasaran.port), penjaga),
        )
        .await;
        let sidik_vps = terlihat.lock().ok().and_then(|t| t.clone());
        let mut handle = match hasil {
            Err(_) => return Err(Galat::BatasWaktu("tersambung ke VPS")),
            Ok(Err(e)) => {
                if let (Some(dikenal), Some(sekarang)) = (sidik_dikenal, sidik_vps)
                    && dikenal != sekarang
                {
                    return Err(Galat::SidikBerubah {
                        dikenal: dikenal.to_owned(),
                        terlihat: sekarang,
                    });
                }
                return Err(sambung(e));
            }
            Ok(Ok(handle)) => handle,
        };
        let sidik =
            sidik_vps.ok_or_else(|| Galat::Sambung("kunci host VPS tidak terbaca".to_owned()))?;
        let masuk_vps = async {
            match masuk {
                Masuk::Sandi(sandi) => {
                    handle
                        .authenticate_password(sasaran.pengguna.as_str(), sandi.as_str())
                        .await
                }
                Masuk::Kunci(kunci) => {
                    handle
                        .authenticate_publickey(
                            sasaran.pengguna.as_str(),
                            PrivateKeyWithHashAlg::new(Arc::clone(kunci), None),
                        )
                        .await
                }
            }
        };
        let hasil = tokio::time::timeout(BATAS_SAMBUNG, masuk_vps)
            .await
            .map_err(|_| Galat::BatasWaktu("masuk ke VPS"))?
            .map_err(sambung)?;
        if !hasil.success() {
            return Err(Galat::MasukDitolak);
        }
        Ok(Self {
            handle,
            sidik,
            root: sasaran.pengguna == "root",
        })
    }

    /// Sidik SHA-256 kunci host VPS (`SHA256:...`, sama dengan `ssh-keygen -l`).
    #[must_use]
    pub fn sidik(&self) -> &str {
        &self.sidik
    }

    /// `true` bila masuk sebagai root (skrip bantu tanpa sudo).
    #[must_use]
    pub const fn root(&self) -> bool {
        self.root
    }

    /// Menjalankan satu baris perintah tetap (dari modul ini) dengan masukan standar opsional.
    pub(super) async fn jalankan(
        &self,
        baris: &str,
        masukan: Option<&[u8]>,
        batas: Duration,
    ) -> Result<Keluaran, Galat> {
        let kerja = async {
            let mut kanal = self.handle.channel_open_session().await.map_err(sambung)?;
            kanal.exec(true, baris).await.map_err(sambung)?;
            if let Some(data) = masukan {
                kanal.data(data).await.map_err(sambung)?;
            }
            kanal.eof().await.map_err(sambung)?;
            let mut keluaran = Keluaran::default();
            while let Some(pesan) = kanal.wait().await {
                match pesan {
                    ChannelMsg::Data { data } => tambah(&mut keluaran.stdout, &data),
                    ChannelMsg::ExtendedData { data, ext: 1 } => {
                        tambah(&mut keluaran.stderr, &data)
                    }
                    ChannelMsg::ExitStatus { exit_status } => keluaran.kode = Some(exit_status),
                    ChannelMsg::Failure => {
                        return Err(Galat::Perintah {
                            kode: None,
                            pesan: "VPS menolak menjalankan perintah".to_owned(),
                        });
                    }
                    _ => {}
                }
            }
            Ok(keluaran)
        };
        tokio::time::timeout(batas, kerja)
            .await
            .map_err(|_| Galat::BatasWaktu("perintah di VPS"))?
    }

    /// Menutup sambungan dengan rapi.
    pub async fn putus(self) {
        let _ = self
            .handle
            .disconnect(Disconnect::ByApplication, "", "id")
            .await;
    }
}
