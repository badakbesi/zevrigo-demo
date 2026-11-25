//! Perintah tetap skrip bantu `demo-zevrigo-kendali` (paket VPS `demo-zevrigo-vps`, Q198).

use std::net::IpAddr;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::pasang::{PAKET_SHA256, hex64};
use super::sesi::{Galat, Keluaran, Sesi};

/// Letak skrip bantu sesudah paket VPS dipasang.
pub const SKRIP_BANTU: &str = "/usr/local/sbin/demo-zevrigo-kendali";

/// Perintah skrip bantu. Hanya perintah ini yang bisa dijalankan aplikasi di VPS; argumen diperiksa sebelum dikirim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kendali {
    /// Keadaan VPS dan layanan (satu baris JSON, [`StatusVps`]).
    Status,
    /// Unduh paket resmi yang dikenal aplikasi ([`PAKET_SHA256`]) lalu pasang; versi lama disimpan.
    Perbarui,
    /// Ganti SHA-256 token pengendali (token lama langsung tidak berlaku).
    Token(String),
    /// Mode node publik dengan IP publik VPS dan port P2P.
    NodePublikNyala {
        /// IP publik VPS.
        ip: IpAddr,
        /// Port P2P (1024 sampai 65535).
        port: u16,
    },
    /// Node kembali menjadi rekan biasa.
    NodePublikMati,
    /// Batas CPU layanan (persen satu CPU, 10 sampai 100).
    BatasCpu(u8),
    /// Matikan login password SSH (hanya bila pengguna SSH sudah punya kunci).
    AmankanSsh,
    /// Hapus kunci publik SSH (bagian base64) dari authorized_keys pengguna SSH.
    CabutKunci(String),
    /// Restart VPS dalam 5 detik.
    RestartVps,
    /// Hentikan dan hapus layanan; data chain disimpan kecuali `hapus_data`.
    Copot {
        /// Hapus juga folder data dan pengguna sistem.
        hapus_data: bool,
    },
}

impl Kendali {
    /// Baris perintah di VPS (`sudo -n` kecuali root).
    ///
    /// # Errors
    ///
    /// [`Galat::Masukan`] bila argumen tidak sah.
    pub fn baris(&self, root: bool) -> Result<String, Galat> {
        let argumen = match self {
            Self::Status => "status".to_owned(),
            Self::Perbarui => format!("perbarui {PAKET_SHA256}"),
            Self::Token(sha) => {
                if !hex64(sha) {
                    return Err(Galat::Masukan("SHA-256 token tidak sah".to_owned()));
                }
                format!("token {sha}")
            }
            Self::NodePublikNyala { ip, port } => {
                let teks = ip.to_string();
                if *port < 1024 || (ip.is_ipv6() && teks.contains('.')) {
                    return Err(Galat::Masukan(
                        "IP atau port node publik tidak sah".to_owned(),
                    ));
                }
                format!("node-publik nyala {teks} {port}")
            }
            Self::NodePublikMati => "node-publik mati".to_owned(),
            Self::BatasCpu(persen) => {
                if !(10..=100).contains(persen) {
                    return Err(Galat::Masukan("batas CPU harus 10 sampai 100".to_owned()));
                }
                format!("batas-cpu {persen}")
            }
            Self::AmankanSsh => "amankan-ssh".to_owned(),
            Self::CabutKunci(blob) => {
                if !blob_kunci_sah(blob) {
                    return Err(Galat::Masukan("kunci publik tidak sah".to_owned()));
                }
                format!("cabut-kunci {blob}")
            }
            Self::RestartVps => "restart-vps".to_owned(),
            Self::Copot { hapus_data: true } => "copot --hapus-data".to_owned(),
            Self::Copot { hapus_data: false } => "copot".to_owned(),
        };
        Ok(if root {
            format!("{SKRIP_BANTU} {argumen}")
        } else {
            format!("sudo -n {SKRIP_BANTU} {argumen}")
        })
    }

    const fn batas(&self) -> Duration {
        match self {
            Self::Perbarui => Duration::from_secs(900),
            Self::Copot { .. } | Self::NodePublikNyala { .. } | Self::NodePublikMati => {
                Duration::from_secs(180)
            }
            _ => Duration::from_secs(60),
        }
    }
}

/// Bagian base64 kunci publik SSH (seperti pemeriksaan skrip bantu `cabut-kunci`).
pub(super) fn blob_kunci_sah(blob: &str) -> bool {
    let isi = blob.trim_end_matches('=');
    blob.len().saturating_sub(isi.len()) <= 2
        && isi.len() >= 40
        && isi
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/')
}

/// Keadaan VPS dari `demo-zevrigo-kendali status`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusVps {
    /// Baris VERSI paket terpasang (kosong bila belum terpasang).
    pub versi: String,
    /// Keadaan layanan (`active`, `inactive`, ...).
    pub layanan: String,
    /// Waktu layanan menyala.
    pub aktif_sejak: String,
    /// Jumlah CPU.
    pub cpu: u32,
    /// Muatan rata-rata 1 menit (teks apa adanya).
    pub muatan1m: String,
    /// RAM total (MB).
    pub ram_total_mb: u64,
    /// RAM tersedia (MB).
    pub ram_tersedia_mb: u64,
    /// Disk total folder data (MB).
    pub disk_total_mb: u64,
    /// Disk bebas folder data (MB).
    pub disk_bebas_mb: u64,
    /// Lama VPS menyala (detik).
    pub uptime_detik: u64,
    /// Alamat node publik (kosong = rekan biasa).
    pub node_publik: String,
    /// Batas CPU layanan per detik (`500ms` = 50%).
    pub kuota_cpu: String,
    /// Login password SSH (`no` = mati).
    pub login_password: String,
}

impl Sesi {
    /// Menjalankan perintah skrip bantu.
    ///
    /// # Errors
    ///
    /// [`Galat::Masukan`], [`Galat::Perintah`] (baris `galat: ...` skrip bantu), galat sambungan.
    pub async fn kendali(&self, perintah: &Kendali) -> Result<Keluaran, Galat> {
        let baris = perintah.baris(self.root())?;
        self.jalankan(&baris, None, perintah.batas())
            .await?
            .wajib_berhasil()
    }

    /// `demo-zevrigo-kendali status`.
    ///
    /// # Errors
    ///
    /// Seperti [`Sesi::kendali`], atau [`Galat::Perintah`] bila JSON tidak terbaca.
    pub async fn status_vps(&self) -> Result<StatusVps, Galat> {
        let keluaran = self.kendali(&Kendali::Status).await?;
        serde_json::from_slice(keluaran.stdout.trim_ascii()).map_err(|e| Galat::Perintah {
            kode: Some(0),
            pesan: format!("status tidak terbaca: {e}"),
        })
    }
}
