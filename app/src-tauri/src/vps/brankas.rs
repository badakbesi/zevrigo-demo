//! Brankas rahasia VPS di HP (DEC-46, Q199b): alamat VPS, sidik kunci host, kunci SSH aplikasi, dan token pengendali,
//! dienkripsi dengan kata sandi aplikasi (Argon2id + XChaCha20-Poly1305, `vault` MOD-09 yang sama dengan Zevrigo Wallet).
//! Tanpa kata sandi tidak ada yang bisa dibaca; lupa kata sandi berarti sambung ulang dengan password VPS (lewat konsol web
//! penyedia VPS), dan node di VPS tetap berjalan.

use std::fmt;

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize as _, Zeroizing};
use zevrigo_wallet_core::vault::{
    KdfParams, NONCE_LEN, SALT_LEN, Sealed, VaultError, seal, unseal,
};

use super::sesi::Galat;

/// Panjang kata sandi aplikasi paling sedikit, dalam karakter (pencipta 26 November 2026).
pub const SANDI_MIN: usize = 8;
/// Panjang kata sandi aplikasi paling banyak, dalam karakter.
pub const SANDI_MAKS: usize = 256;
/// Jenis berkas brankas.
const JENIS: &str = "zevrigo-node-vps";
/// Versi format berkas brankas.
const VERSI: u32 = 1;

/// Isi brankas (hanya ada di memori selama aplikasi terbuka; kunci dan token dihapus dari memori saat dibuang).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rahasia {
    /// Host VPS.
    pub host: String,
    /// Port SSH.
    pub port: u16,
    /// Pengguna SSH.
    pub pengguna: String,
    /// Sidik SHA-256 kunci host VPS yang dipercaya pada sambungan pertama.
    pub sidik: String,
    /// Kunci SSH ed25519 aplikasi (format OpenSSH).
    pub kunci: String,
    /// Token pengendali.
    pub token: String,
    /// Login password SSH VPS sudah dimatikan aplikasi (`amankan-ssh`).
    pub amankan_selesai: bool,
}

impl fmt::Debug for Rahasia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rahasia")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("pengguna", &self.pengguna)
            .field("sidik", &self.sidik)
            .field("amankan_selesai", &self.amankan_selesai)
            .finish_non_exhaustive()
    }
}

impl Drop for Rahasia {
    fn drop(&mut self) {
        self.kunci.zeroize();
        self.token.zeroize();
    }
}

/// Berkas brankas (JSON). Tidak ada yang rahasia di sini: isi terenkripsi.
#[derive(Serialize, Deserialize)]
struct Berkas {
    jenis: String,
    versi: u32,
    memori_kib: u32,
    putaran: u32,
    jalur: u32,
    salt: String,
    nonce: String,
    isi: String,
}

/// `true` bila panjang kata sandi aplikasi [`SANDI_MIN`] sampai [`SANDI_MAKS`] karakter.
#[must_use]
pub fn sandi_sah(sandi: &str) -> bool {
    (SANDI_MIN..=SANDI_MAKS).contains(&sandi.chars().count())
}

fn sistem(e: impl fmt::Display) -> Galat {
    Galat::Sistem(e.to_string())
}

/// Mengenkripsi `rahasia` dengan `sandi`. Hasilnya teks berkas brankas, aman disimpan atau diekspor (tanpa sandi tidak
/// terbaca).
///
/// # Errors
///
/// [`Galat::Masukan`] bila sandi tidak memenuhi syarat panjang, [`Galat::Sistem`] bila sumber acak atau enkripsi gagal.
pub fn tutup_brankas(rahasia: &Rahasia, sandi: &str, kdf: KdfParams) -> Result<String, Galat> {
    if !sandi_sah(sandi) {
        return Err(Galat::Masukan(format!(
            "kata sandi aplikasi {SANDI_MIN} sampai {SANDI_MAKS} karakter"
        )));
    }
    let isi = Zeroizing::new(serde_json::to_vec(rahasia).map_err(sistem)?);
    let mut salt = [0_u8; SALT_LEN];
    let mut nonce = [0_u8; NONCE_LEN];
    getrandom::fill(&mut salt).map_err(sistem)?;
    getrandom::fill(&mut nonce).map_err(sistem)?;
    let tertutup = seal(&isi, sandi.as_bytes(), kdf, salt, nonce).map_err(sistem)?;
    let berkas = Berkas {
        jenis: JENIS.to_owned(),
        versi: VERSI,
        memori_kib: tertutup.kdf.memory_kib,
        putaran: tertutup.kdf.iterations,
        jalur: tertutup.kdf.parallelism,
        salt: hex::encode(tertutup.salt),
        nonce: hex::encode(tertutup.nonce),
        isi: hex::encode(&tertutup.ciphertext),
    };
    serde_json::to_string_pretty(&berkas).map_err(sistem)
}

/// Membuka teks berkas brankas dengan `sandi`.
///
/// # Errors
///
/// [`Galat::SandiSalah`] bila sandi salah atau isi diubah, [`Galat::Brankas`] bila teks bukan berkas brankas yang dikenal.
pub fn buka_brankas(teks: &str, sandi: &str) -> Result<Rahasia, Galat> {
    let rusak = |pesan: &str| Galat::Brankas(pesan.to_owned());
    let berkas: Berkas =
        serde_json::from_str(teks.trim()).map_err(|_| rusak("bukan data VPS Zevrigo Node"))?;
    if berkas.jenis != JENIS || berkas.versi != VERSI {
        return Err(rusak("jenis atau versi data VPS tidak dikenal"));
    }
    let salt: [u8; SALT_LEN] = hex::decode(&berkas.salt)
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or_else(|| rusak("salt data VPS rusak"))?;
    let nonce: [u8; NONCE_LEN] = hex::decode(&berkas.nonce)
        .ok()
        .and_then(|b| b.try_into().ok())
        .ok_or_else(|| rusak("nonce data VPS rusak"))?;
    let ciphertext = hex::decode(&berkas.isi).map_err(|_| rusak("isi data VPS rusak"))?;
    let tertutup = Sealed {
        kdf: KdfParams {
            memory_kib: berkas.memori_kib,
            iterations: berkas.putaran,
            parallelism: berkas.jalur,
        },
        salt,
        nonce,
        ciphertext,
    };
    let isi = unseal(&tertutup, sandi.as_bytes()).map_err(|e| match e {
        VaultError::Decrypt => Galat::SandiSalah,
        lain => Galat::Brankas(lain.to_string()),
    })?;
    serde_json::from_slice(&isi).map_err(|_| rusak("isi data VPS tidak terbaca"))
}
