//! Pemeriksaan dan pemasangan pertama di VPS, kunci SSH aplikasi, dan token pengendali.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use russh::keys::ssh_key::LineEnding;
use russh::keys::ssh_key::private::Ed25519Keypair;
use russh::keys::{Algorithm, PrivateKey};
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use zeroize::Zeroizing;

use super::kendali::SKRIP_BANTU;
use super::sesi::{Galat, Keluaran, Masuk, Sesi};

/// Rilis resmi paket VPS (tag tetap `demo-zevrigo-vps`, DEC-46).
pub const RILIS: &str =
    "https://github.com/badakbesi/zevrigo-demo/releases/download/demo-zevrigo-vps";
/// Nama berkas paket VPS di rilis.
pub const PAKET: &str = "demo-zevrigo-vps-linux-x86_64.tar.gz";
/// SHA-256 paket VPS yang dikenal aplikasi ini (tertanam di aplikasi, pencipta 25 November 2026): rilis `demo-zevrigo-vps`
/// zevrigo-demo faa9c3a (Q198f, Q198g). Paket VPS baru berarti versi aplikasi baru.
pub const PAKET_SHA256: &str = "7d90a27fc6e7cd67eda944291a77fdb9cf1fc13ba7b0765a3e1a4cffb333306b";
/// Komentar kunci publik aplikasi di authorized_keys.
const KOMENTAR_KUNCI: &str = "zevrigo-node-android";
/// Skrip pemeriksaan VPS (tanpa sudo, tanpa mengubah apa pun).
const SKRIP_PERIKSA: &str = include_str!("periksa.sh");
/// Skrip pemasangan (dijalankan sebagai root).
const SKRIP_PASANG: &str = include_str!("pasang.sh");
/// Letak skrip pemasangan sementara di folder rumah pengguna SSH (skrip menghapus dirinya sendiri).
pub(super) const BERKAS_PASANG: &str = ".cache/demo-zevrigo-pasang.sh";

/// `true` bila teks adalah SHA-256 dalam 64 huruf hex kecil.
pub(super) fn hex64(teks: &str) -> bool {
    teks.len() == 64 && teks.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// SHA-256 (hex kecil) dari teks, misalnya token pengendali (VPS hanya menyimpan nilai ini).
#[must_use]
pub fn sha256_hex(teks: &str) -> String {
    hex::encode(Sha256::digest(teks.as_bytes()))
}

fn acak(tujuan: &mut [u8]) -> Result<(), Galat> {
    getrandom::fill(tujuan).map_err(|e| Galat::Sistem(format!("sumber acak sistem: {e}")))
}

/// Token pengendali baru: 64 huruf/angka acak (A-Z, a-z, 0-9) dari sumber acak sistem.
///
/// # Errors
///
/// [`Galat::Sistem`] bila sumber acak sistem gagal.
pub fn buat_token() -> Result<Zeroizing<String>, Galat> {
    const HURUF: &[u8; 62] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut token = Zeroizing::new(String::with_capacity(64));
    while token.len() < 64 {
        let mut bahan = Zeroizing::new([0_u8; 64]);
        acak(bahan.as_mut_slice())?;
        for b in bahan.iter() {
            // 248 = 4 x 62: tanpa bias.
            if *b < 248
                && token.len() < 64
                && let Some(&c) = HURUF.get(usize::from(*b % 62))
            {
                token.push(char::from(c));
            }
        }
    }
    Ok(token)
}

/// Kunci SSH ed25519 aplikasi (disimpan aplikasi di vault kata sandi, DEC-46).
#[derive(Clone)]
pub struct Kunci(Arc<PrivateKey>);

impl fmt::Debug for Kunci {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Kunci(..)")
    }
}

impl Kunci {
    /// Kunci baru dari sumber acak sistem.
    ///
    /// # Errors
    ///
    /// [`Galat::Sistem`] bila sumber acak sistem gagal.
    pub fn baru() -> Result<Self, Galat> {
        let mut benih = Zeroizing::new([0_u8; 32]);
        acak(benih.as_mut_slice())?;
        let mut kunci = PrivateKey::from(Ed25519Keypair::from_seed(&benih));
        kunci.set_comment(KOMENTAR_KUNCI);
        Ok(Self(Arc::new(kunci)))
    }

    /// Kunci dari teks OpenSSH yang disimpan aplikasi.
    ///
    /// # Errors
    ///
    /// [`Galat::Masukan`] bila teks bukan kunci ed25519 tanpa sandi.
    pub fn dari_openssh(teks: &str) -> Result<Self, Galat> {
        let kunci = PrivateKey::from_openssh(teks)
            .map_err(|e| Galat::Masukan(format!("kunci SSH tidak terbaca: {e}")))?;
        if kunci.algorithm() != Algorithm::Ed25519 || kunci.is_encrypted() {
            return Err(Galat::Masukan(
                "kunci SSH harus ed25519 tanpa sandi".to_owned(),
            ));
        }
        Ok(Self(Arc::new(kunci)))
    }

    /// Kunci rahasia dalam format OpenSSH (untuk disimpan di vault aplikasi).
    ///
    /// # Errors
    ///
    /// [`Galat::Sistem`] bila penyandian gagal.
    pub fn ke_openssh(&self) -> Result<Zeroizing<String>, Galat> {
        self.0
            .to_openssh(LineEnding::LF)
            .map_err(|e| Galat::Sistem(e.to_string()))
    }

    /// Baris kunci publik untuk authorized_keys (`ssh-ed25519 AAAA... zevrigo-node-android`).
    ///
    /// # Errors
    ///
    /// [`Galat::Sistem`] bila penyandian gagal.
    pub fn baris_publik(&self) -> Result<String, Galat> {
        self.0
            .public_key()
            .to_openssh()
            .map_err(|e| Galat::Sistem(e.to_string()))
    }

    /// Bagian base64 kunci publik (untuk [`super::Kendali::CabutKunci`]).
    ///
    /// # Errors
    ///
    /// [`Galat::Sistem`] bila penyandian gagal.
    pub fn blob_publik(&self) -> Result<String, Galat> {
        let baris = self.baris_publik()?;
        baris
            .split(' ')
            .nth(1)
            .map(str::to_owned)
            .ok_or_else(|| Galat::Sistem("kunci publik tanpa bagian base64".to_owned()))
    }

    /// Cara masuk ke VPS dengan kunci ini.
    #[must_use]
    pub fn masuk(&self) -> Masuk {
        Masuk::Kunci(Arc::clone(&self.0))
    }
}

/// Hasil pemeriksaan VPS sebelum pemasangan.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PemeriksaanVps {
    /// `uname -m`.
    pub arsitektur: String,
    /// ID sistem (`ubuntu`, `debian`, ...).
    pub os: String,
    /// Versi sistem (`22.04`, `12`, ...).
    pub versi_os: String,
    /// Nama sistem untuk ditampilkan.
    pub nama_os: String,
    /// systemd berjalan.
    pub systemd: bool,
    /// Pengguna SSH adalah root.
    pub root: bool,
    /// Perintah sudo ada.
    pub sudo: bool,
    /// curl atau wget ada.
    pub unduh: bool,
    /// Daftar sertifikat CA sistem ada.
    pub ca: bool,
    /// Zevrigo Node sudah terpasang.
    pub terpasang: bool,
    /// Disk bebas di `/` (MB).
    pub disk_bebas_mb: u64,
    /// RAM total (MB).
    pub ram_mb: u64,
}

impl PemeriksaanVps {
    /// Membaca baris `kunci=nilai` keluaran skrip pemeriksaan.
    pub(super) fn dari_teks(teks: &str) -> Self {
        let mut hasil = Self::default();
        for baris in teks.lines() {
            let Some((kunci, nilai)) = baris.split_once('=') else {
                continue;
            };
            let nilai = nilai.trim();
            let ya = nilai == "ya";
            match kunci.trim() {
                "arsitektur" => nilai.clone_into(&mut hasil.arsitektur),
                "os" => nilai.clone_into(&mut hasil.os),
                "versi_os" => nilai.clone_into(&mut hasil.versi_os),
                "nama_os" => nilai.clone_into(&mut hasil.nama_os),
                "systemd" => hasil.systemd = ya,
                "root" => hasil.root = ya,
                "sudo" => hasil.sudo = ya,
                "unduh" => hasil.unduh = ya,
                "ca" => hasil.ca = ya,
                "terpasang" => hasil.terpasang = ya,
                "disk_bebas_mb" => hasil.disk_bebas_mb = nilai.parse().unwrap_or(0),
                "ram_mb" => hasil.ram_mb = nilai.parse().unwrap_or(0),
                _ => {}
            }
        }
        hasil
    }

    /// Alasan VPS ini tidak bisa dipasangi Zevrigo Node (kosong = bisa; syarat sama dengan paket VPS).
    #[must_use]
    pub fn masalah(&self) -> Vec<String> {
        let mut masalah = Vec::new();
        if self.arsitektur != "x86_64" {
            masalah.push(format!("VPS bukan x86_64 ({})", self.arsitektur));
        }
        let utama: u32 = self
            .versi_os
            .split('.')
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let didukung = match self.os.as_str() {
            "ubuntu" => utama >= 22,
            "debian" => utama >= 12,
            _ => false,
        };
        if !didukung {
            masalah.push(format!(
                "sistem {} {} belum didukung (butuh Ubuntu 22.04 atau lebih baru, atau Debian 12 atau lebih baru)",
                self.os, self.versi_os
            ));
        }
        if !self.systemd {
            masalah.push("systemd tidak berjalan".to_owned());
        }
        if !self.root && !self.sudo {
            masalah.push("butuh pengguna root atau perintah sudo".to_owned());
        }
        if !self.unduh {
            masalah.push("curl atau wget tidak ada".to_owned());
        }
        if !self.ca {
            masalah.push("daftar sertifikat CA tidak ada (paket ca-certificates)".to_owned());
        }
        if self.disk_bebas_mb < 2048 {
            masalah.push(format!(
                "disk bebas {} MB, butuh paling sedikit 2048 MB",
                self.disk_bebas_mb
            ));
        }
        if self.ram_mb < 700 {
            masalah.push(format!("RAM {} MB, butuh paling sedikit 1 GB", self.ram_mb));
        }
        masalah
    }
}

/// Memeriksa VPS (sebagai pengguna SSH, tanpa sudo, tanpa mengubah apa pun).
///
/// # Errors
///
/// Galat sambungan atau [`Galat::Perintah`].
pub async fn periksa_vps(sesi: &Sesi) -> Result<PemeriksaanVps, Galat> {
    let keluaran = sesi
        .jalankan(
            "sh -s",
            Some(SKRIP_PERIKSA.as_bytes()),
            Duration::from_secs(60),
        )
        .await?
        .wajib_berhasil()?;
    Ok(PemeriksaanVps::dari_teks(&keluaran.teks()))
}

/// Pemasangan pertama sebagai root (sudo dengan password pengguna, atau langsung bila root): VPS mengunduh paket dari
/// [`RILIS`], mencocokkan [`PAKET_SHA256`], lalu `demo-zevrigo-kendali pasang` dengan SHA-256 token aplikasi. Mengembalikan baris
/// `terpasang: ...` skrip bantu.
///
/// # Errors
///
/// [`Galat::Masukan`] (VPS tidak memenuhi syarat, sudah terpasang, password sudo salah atau tidak boleh sudo),
/// [`Galat::Perintah`], galat sambungan.
pub async fn pasang_baru(
    sesi: &Sesi,
    sandi_sudo: Option<&str>,
    token_sha256: &str,
) -> Result<String, Galat> {
    if !hex64(token_sha256) {
        return Err(Galat::Masukan("SHA-256 token tidak sah".to_owned()));
    }
    let pemeriksaan = periksa_vps(sesi).await?;
    let masalah = pemeriksaan.masalah();
    if !masalah.is_empty() {
        return Err(Galat::Masukan(masalah.join("; ")));
    }
    if pemeriksaan.terpasang {
        return Err(Galat::Masukan(
            "Zevrigo Node sudah terpasang di VPS ini".to_owned(),
        ));
    }
    let kirim =
        format!("umask 077 && mkdir -p \"$HOME/.cache\" && cat > \"$HOME/{BERKAS_PASANG}\"");
    sesi.jalankan(
        &kirim,
        Some(SKRIP_PASANG.as_bytes()),
        Duration::from_secs(60),
    )
    .await?
    .wajib_berhasil()?;
    let jalan = format!("bash \"$HOME/{BERKAS_PASANG}\" {token_sha256} {PAKET_SHA256}");
    let (baris, masukan) = dengan_sudo(sesi, sandi_sudo, &jalan)?;
    let hasil = sesi
        .jalankan(
            &baris,
            masukan.as_ref().map(|s| s.as_bytes()),
            Duration::from_secs(1200),
        )
        .await;
    // Skrip menghapus dirinya sendiri; bila sudo gagal, skrip tidak berjalan dan dihapus di sini.
    let hapus = format!("rm -f \"$HOME/{BERKAS_PASANG}\"");
    let _ = sesi.jalankan(&hapus, None, Duration::from_secs(30)).await;
    let keluaran = periksa_sudo(hasil?)?.wajib_berhasil()?;
    Ok(keluaran
        .teks()
        .lines()
        .rev()
        .find(|b| b.starts_with("terpasang: "))
        .unwrap_or("terpasang")
        .to_owned())
}

/// Baris perintah sebagai root: langsung bila pengguna SSH root, selain itu `sudo -S` dengan password pengguna di stdin.
fn dengan_sudo(
    sesi: &Sesi,
    sandi_sudo: Option<&str>,
    perintah: &str,
) -> Result<(String, Option<Zeroizing<String>>), Galat> {
    if sesi.root() {
        return Ok((perintah.to_owned(), None));
    }
    let sandi = sandi_sudo.ok_or_else(|| Galat::Masukan("password sudo dibutuhkan".to_owned()))?;
    if sandi.is_empty() || sandi.contains(['\n', '\r']) {
        return Err(Galat::Masukan("password sudo tidak sah".to_owned()));
    }
    Ok((
        format!("sudo -S -p '' {perintah}"),
        Some(Zeroizing::new(format!("{sandi}\n"))),
    ))
}

/// Galat sudo yang dikenal menjadi pesan untuk pengguna.
fn periksa_sudo(keluaran: Keluaran) -> Result<Keluaran, Galat> {
    if !keluaran.berhasil() {
        let galat = String::from_utf8_lossy(&keluaran.stderr);
        if galat.contains("incorrect password") || galat.contains("Sorry, try again") {
            return Err(Galat::Masukan("password sudo salah".to_owned()));
        }
        if galat.contains("not in the sudoers") || galat.contains("may not run sudo") {
            return Err(Galat::Masukan(
                "pengguna ini tidak boleh memakai sudo".to_owned(),
            ));
        }
    }
    Ok(keluaran)
}

/// VPS yang sudah terpasang Zevrigo Node (aplikasi dipasang ulang, HP baru tanpa data lama, atau kata sandi aplikasi lupa):
/// token pengendali diganti dengan token HP ini lewat skrip bantu (sudo dengan password pengguna, atau langsung bila root).
/// Node, penambang, dan data chain tidak disentuh.
///
/// # Errors
///
/// [`Galat::Masukan`] (SHA-256 token tidak sah, password sudo salah), [`Galat::Perintah`], galat sambungan.
pub async fn ambil_alih(
    sesi: &Sesi,
    sandi_sudo: Option<&str>,
    token_sha256: &str,
) -> Result<(), Galat> {
    if !hex64(token_sha256) {
        return Err(Galat::Masukan("SHA-256 token tidak sah".to_owned()));
    }
    let perintah = format!("{SKRIP_BANTU} token {token_sha256}");
    let (baris, masukan) = dengan_sudo(sesi, sandi_sudo, &perintah)?;
    let keluaran = sesi
        .jalankan(
            &baris,
            masukan.as_ref().map(|s| s.as_bytes()),
            Duration::from_secs(60),
        )
        .await?;
    periksa_sudo(keluaran)?.wajib_berhasil()?;
    Ok(())
}

/// Memasang kunci publik aplikasi di authorized_keys pengguna SSH (tanpa sudo; tidak menggandakan baris yang sama).
///
/// # Errors
///
/// [`Galat::Sistem`] bila kunci tidak bisa disandikan, [`Galat::Perintah`], galat sambungan.
pub async fn pasang_kunci_publik(sesi: &Sesi, kunci: &Kunci) -> Result<(), Galat> {
    let baris = kunci.baris_publik()?;
    if !baris
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '+' | '/' | '=' | '-'))
    {
        return Err(Galat::Sistem("baris kunci publik tidak terduga".to_owned()));
    }
    let ak = "\"$HOME/.ssh/authorized_keys\"";
    let perintah = format!(
        "umask 077 && mkdir -p \"$HOME/.ssh\" && touch {ak} && chmod 700 \"$HOME/.ssh\" && chmod 600 {ak} && {{ grep -qxF '{baris}' {ak} || {{ [ -z \"$(tail -c 1 {ak})\" ] || echo >> {ak}; printf '%s\\n' '{baris}' >> {ak}; }}; }}"
    );
    sesi.jalankan(&perintah, None, Duration::from_secs(60))
        .await?
        .wajib_berhasil()?;
    Ok(())
}
