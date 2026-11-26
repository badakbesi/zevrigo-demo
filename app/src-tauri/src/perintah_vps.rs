//! Perintah IPC Tauri aplikasi Zevrigo Node mode VPS (fitur `vps`: Android, RENCANA_T17 bagian 13; Q199b). Antarmuka:
//! `src/lib/vps.ts`. Kata sandi aplikasi, password VPS, kunci SSH, dan token pengendali tidak pernah dikirim ke antarmuka;
//! yang keluar hanya keadaan tanpa rahasia dan teks brankas terenkripsi untuk ekspor.

use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;
use tauri::State;
use tauri_plugin_opener::OpenerExt as _;
use zevrigo_node_mesin::tautan_resmi;

use crate::vps::{Galat, InfoLayanan, Layanan, PerintahUi, StatusVps};

/// Galat untuk antarmuka: kode singkat (`Galat::kode`) dan pesan.
#[derive(Clone, Debug, Serialize)]
pub struct GalatPerintah {
    kode: &'static str,
    pesan: String,
}

impl From<Galat> for GalatPerintah {
    fn from(galat: Galat) -> Self {
        Self {
            kode: galat.kode(),
            pesan: galat.to_string(),
        }
    }
}

type Hasil<T> = Result<T, GalatPerintah>;

/// Pekerjaan berat (Argon2id) di utas sendiri.
async fn berat<T, F>(job: F) -> Hasil<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, Galat> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(job)
        .await
        .map_err(|_| {
            GalatPerintah::from(Galat::Sistem("pekerjaan berhenti tak terduga".to_owned()))
        })?
        .map_err(GalatPerintah::from)
}

/// `mode_aplikasi`: `vps` (node di VPS pengguna, dikendalikan lewat SSH).
#[tauri::command]
pub fn mode_aplikasi() -> &'static str {
    "vps"
}

/// `vps_keadaan`.
#[tauri::command]
pub async fn vps_keadaan(state: State<'_, Arc<Layanan>>) -> Hasil<InfoLayanan> {
    Ok(state.info()?)
}

/// `vps_buat_sandi`: kata sandi aplikasi baru (tahap baru).
#[tauri::command]
pub async fn vps_buat_sandi(state: State<'_, Arc<Layanan>>, sandi: String) -> Hasil<InfoLayanan> {
    Ok(state.buat_sandi(&sandi)?)
}

/// `vps_batal`: batalkan pemasangan pertama.
#[tauri::command]
pub async fn vps_batal(state: State<'_, Arc<Layanan>>) -> Hasil<InfoLayanan> {
    Ok(state.batal().await?)
}

/// `vps_sambung`: sambungan pertama dengan password VPS.
#[tauri::command]
pub async fn vps_sambung(
    state: State<'_, Arc<Layanan>>,
    host: String,
    port: u16,
    pengguna: String,
    sandi: String,
) -> Hasil<InfoLayanan> {
    Ok(state.sambung_awal(&host, port, &pengguna, &sandi).await?)
}

/// `vps_pasang`: pasang paket VPS (atau ambil alih node yang sudah ada).
#[tauri::command]
pub async fn vps_pasang(state: State<'_, Arc<Layanan>>) -> Hasil<InfoLayanan> {
    Ok(state.pasang().await?)
}

/// `vps_amankan`: matikan login password SSH VPS.
#[tauri::command]
pub async fn vps_amankan(state: State<'_, Arc<Layanan>>) -> Hasil<InfoLayanan> {
    Ok(state.amankan().await?)
}

/// `vps_lewati_amankan`.
#[tauri::command]
pub async fn vps_lewati_amankan(state: State<'_, Arc<Layanan>>) -> Hasil<InfoLayanan> {
    Ok(state.lewati_amankan()?)
}

/// `vps_buka`: buka aplikasi dengan kata sandi aplikasi.
#[tauri::command]
pub async fn vps_buka(state: State<'_, Arc<Layanan>>, sandi: String) -> Hasil<InfoLayanan> {
    let layanan = Arc::clone(state.inner());
    berat(move || layanan.buka(&sandi)).await
}

/// `vps_kunci`: kunci aplikasi (node di VPS tetap berjalan).
#[tauri::command]
pub async fn vps_kunci(state: State<'_, Arc<Layanan>>) -> Hasil<InfoLayanan> {
    Ok(state.kunci().await?)
}

/// `vps_rpc`: metode pengendali (daftar tetap di `vps::METODE`).
#[tauri::command]
pub async fn vps_rpc(
    state: State<'_, Arc<Layanan>>,
    metode: String,
    params: Value,
) -> Hasil<Value> {
    Ok(state.rpc(&metode, params).await?)
}

/// `vps_status`: keadaan VPS dari skrip bantu.
#[tauri::command]
pub async fn vps_status(state: State<'_, Arc<Layanan>>) -> Hasil<StatusVps> {
    Ok(state.status().await?)
}

/// `vps_kendali`: perintah pemeliharaan (perbarui, batas CPU, node publik, restart VPS).
#[tauri::command]
pub async fn vps_kendali(state: State<'_, Arc<Layanan>>, perintah: PerintahUi) -> Hasil<String> {
    let kendali = perintah.ke_kendali()?;
    Ok(state.kendali(&kendali).await?.teks())
}

/// `vps_ganti_token`.
#[tauri::command]
pub async fn vps_ganti_token(state: State<'_, Arc<Layanan>>) -> Hasil<()> {
    Ok(state.ganti_token().await?)
}

/// `vps_ekspor`: teks brankas terenkripsi untuk pindah HP.
#[tauri::command]
pub async fn vps_ekspor(state: State<'_, Arc<Layanan>>) -> Hasil<String> {
    Ok(state.ekspor()?)
}

/// `vps_impor`: data VPS dari HP lama.
#[tauri::command]
pub async fn vps_impor(
    state: State<'_, Arc<Layanan>>,
    teks: String,
    sandi: String,
) -> Hasil<InfoLayanan> {
    let layanan = Arc::clone(state.inner());
    berat(move || layanan.impor(&teks, &sandi)).await
}

/// `vps_lupakan`: hapus data VPS dari HP ini.
#[tauri::command]
pub async fn vps_lupakan(state: State<'_, Arc<Layanan>>) -> Hasil<InfoLayanan> {
    Ok(state.lupakan().await?)
}

/// `vps_cabut_akses`: cabut kunci SSH HP ini dari VPS, lalu hapus data VPS dari HP.
#[tauri::command]
pub async fn vps_cabut_akses(state: State<'_, Arc<Layanan>>) -> Hasil<InfoLayanan> {
    Ok(state.cabut_akses().await?)
}

/// `vps_copot`: copot Zevrigo Node dari VPS, lalu hapus data VPS dari HP.
#[tauri::command]
pub async fn vps_copot(state: State<'_, Arc<Layanan>>, hapus: bool) -> Hasil<InfoLayanan> {
    Ok(state.copot(hapus).await?)
}

/// `buka_tautan`: hanya tautan resmi di browser sistem.
#[tauri::command]
pub async fn buka_tautan(app: tauri::AppHandle, url: String) -> Hasil<()> {
    if !tautan_resmi(&url) {
        return Err(Galat::Masukan("tautan bukan tautan resmi Zevrigo".to_owned()).into());
    }
    app.opener()
        .open_url(url.as_str(), None::<&str>)
        .map_err(|e| Galat::Sistem(format!("browser tidak bisa dibuka: {e}")).into())
}
