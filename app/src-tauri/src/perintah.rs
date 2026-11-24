//! Perintah IPC Tauri untuk antarmuka (kontrak `Backend` di src/lib/backend.ts). Pekerjaan yang bisa menunggu (menyalakan dan
//! menghentikan node, menghentikan penambang) berjalan di utas sendiri tanpa konteks runtime Tauri, karena MOD-12 memakai
//! runtime tokio sendiri (pola wallet MOD-15).

use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use tauri_plugin_opener::OpenerExt as _;

use crate::mesin::{
    BarisLog, HasilPeriksaAlamat, KeadaanNode, KeadaanTambang, Mesin, Pengaturan,
    periksa_alamat as periksa, tautan_resmi,
};

type Hasil<T> = Result<T, String>;

/// Versi aplikasi dan node.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoAplikasi {
    versi: String,
    versi_node: String,
}

/// Menjalankan `job` di utas sendiri.
async fn run_blocking<T, F>(job: F) -> Hasil<T>
where
    T: Send + 'static,
    F: FnOnce() -> Hasil<T> + Send + 'static,
{
    let berhenti = || "pekerjaan berhenti tak terduga".to_owned();
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::spawn(job)
            .join()
            .unwrap_or_else(|_| Err(berhenti()))
    })
    .await
    .map_err(|_| berhenti())?
}

/// `info`.
#[tauri::command]
pub async fn info(state: State<'_, Arc<Mesin>>) -> Hasil<InfoAplikasi> {
    let mesin = Arc::clone(state.inner());
    run_blocking(move || {
        Ok(InfoAplikasi {
            versi: env!("CARGO_PKG_VERSION").to_owned(),
            versi_node: mesin.versi_node(),
        })
    })
    .await
}

/// `muat_pengaturan`.
#[tauri::command]
pub async fn muat_pengaturan(state: State<'_, Arc<Mesin>>) -> Hasil<Pengaturan> {
    Ok(state.muat_pengaturan())
}

/// `simpan_pengaturan`.
#[tauri::command]
pub async fn simpan_pengaturan(state: State<'_, Arc<Mesin>>, pengaturan: Pengaturan) -> Hasil<()> {
    state.simpan_pengaturan(pengaturan)
}

/// `nyalakan_node`.
#[tauri::command]
pub async fn nyalakan_node(state: State<'_, Arc<Mesin>>) -> Hasil<()> {
    let mesin = Arc::clone(state.inner());
    run_blocking(move || mesin.nyalakan_node()).await
}

/// `hentikan_node` (penambang ikut dihentikan).
#[tauri::command]
pub async fn hentikan_node(state: State<'_, Arc<Mesin>>) -> Hasil<()> {
    let mesin = Arc::clone(state.inner());
    run_blocking(move || {
        mesin.hentikan_node();
        Ok(())
    })
    .await
}

/// `keadaan_node`.
#[tauri::command]
pub async fn keadaan_node(state: State<'_, Arc<Mesin>>) -> Hasil<KeadaanNode> {
    Ok(state.keadaan_node())
}

/// `periksa_alamat`: hanya hasil sah atau alasan; payload tidak dikirim ke antarmuka.
#[tauri::command]
pub async fn periksa_alamat(alamat: String) -> Hasil<HasilPeriksaAlamat> {
    Ok(periksa(&alamat).0)
}

/// `mulai_tambang`.
#[tauri::command]
pub async fn mulai_tambang(state: State<'_, Arc<Mesin>>, alamat: String, jeda: u64) -> Hasil<()> {
    state.mulai_tambang(&alamat, jeda)
}

/// `berhenti_tambang`.
#[tauri::command]
pub async fn berhenti_tambang(state: State<'_, Arc<Mesin>>) -> Hasil<()> {
    let mesin = Arc::clone(state.inner());
    run_blocking(move || {
        mesin.hentikan_tambang();
        Ok(())
    })
    .await
}

/// `keadaan_tambang`.
#[tauri::command]
pub async fn keadaan_tambang(state: State<'_, Arc<Mesin>>) -> Hasil<KeadaanTambang> {
    Ok(state.keadaan_tambang())
}

/// `log_terbaru`: baris sesudah waktu `sejak` (milidetik), atau semua.
#[tauri::command]
pub async fn log_terbaru(state: State<'_, Arc<Mesin>>, sejak: Option<u64>) -> Hasil<Vec<BarisLog>> {
    Ok(state.log_sejak(sejak))
}

/// `buka_folder_data`: folder data di pengelola berkas sistem.
#[tauri::command]
pub async fn buka_folder_data(app: tauri::AppHandle, state: State<'_, Arc<Mesin>>) -> Hasil<()> {
    let folder = state.folder().to_path_buf();
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(folder.display().to_string(), None::<&str>)
        .map_err(|e| format!("folder data tidak bisa dibuka: {e}"))
}

/// `buka_tautan`: hanya tautan resmi ([`tautan_resmi`]) di browser sistem.
#[tauri::command]
pub async fn buka_tautan(
    app: tauri::AppHandle,
    state: State<'_, Arc<Mesin>>,
    url: String,
) -> Hasil<()> {
    if !tautan_resmi(&url) {
        state.catat(
            "aplikasi",
            &format!("tautan bukan tautan resmi, tidak dibuka: {url}"),
        );
        return Err("tautan bukan tautan resmi Zevrigo".to_owned());
    }
    app.opener()
        .open_url(url.as_str(), None::<&str>)
        .map_err(|e| format!("browser tidak bisa dibuka: {e}"))
}

/// `akan_tutup`: `true` sesudah pengguna menutup jendela saat node atau penambang berjalan.
#[tauri::command]
pub async fn akan_tutup(state: State<'_, Arc<Mesin>>) -> Hasil<bool> {
    Ok(state.akan_tutup())
}

/// `selesai_tutup`: menghentikan semua proses lalu keluar.
#[tauri::command]
pub async fn selesai_tutup(app: tauri::AppHandle, state: State<'_, Arc<Mesin>>) -> Hasil<()> {
    let mesin = Arc::clone(state.inner());
    run_blocking(move || {
        mesin.hentikan_semua();
        Ok(())
    })
    .await?;
    app.exit(0);
    Ok(())
}
