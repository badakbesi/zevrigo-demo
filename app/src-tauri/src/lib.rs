//! Zevrigo Node (RENCANA_T16, DEC-44): aplikasi untuk peserta jaringan demo Zevrigo (regtest).
//!
//! Desktop (tanpa fitur `vps`): antarmuka React (`../src`) memanggil perintah IPC di `perintah`. Node dan penambang adalah
//! program `zevrigo-node` dan `zevrigo-miner` dari repo inti yang dibawa sebagai sidecar (Tauri `externalBin`) dan dijalankan
//! sebagai proses anak oleh `mesin` (crate `zevrigo-node-mesin` di zevrigo-demo/mesin, DEC-45). Folder data sama dengan paket
//! peserta (`~/.zevrigo`, `%USERPROFILE%\.zevrigo`, atau `ZVRG_DATA`). Aplikasi tidak membuat dan tidak menyimpan kunci
//! rahasia wallet: hadiah tambang ke alamat Zevrigo Wallet yang ditempel pengguna.
//!
//! Android (fitur `vps`, RENCANA_T17, DEC-46): node dan penambang di VPS pengguna, dikendalikan lewat SSH (`vps`,
//! perintah IPC di `perintah_vps`). Kunci SSH dan token pengendali disimpan di HP dalam brankas kata sandi aplikasi.

#![forbid(unsafe_code)]

#[cfg(not(feature = "vps"))]
mod perintah;
#[cfg(feature = "vps")]
mod perintah_vps;
/// Klien VPS untuk aplikasi Android (DEC-46, Q199a, Q199b); hanya dengan fitur `vps`.
#[cfg(feature = "vps")]
pub mod vps;

use std::sync::Arc;
#[cfg(not(feature = "vps"))]
use std::time::Duration;

use tauri::Manager as _;
#[cfg(not(feature = "vps"))]
use zevrigo_node_mesin as mesin;

/// Batas waktu antarmuka menghentikan node dengan rapi sesudah jendela ditutup; sesudah itu aplikasi menghentikannya sendiri.
#[cfg(not(feature = "vps"))]
const BATAS_TUTUP: Duration = Duration::from_secs(45);

/// Menjalankan aplikasi sampai jendelanya ditutup. Menutup jendela saat node atau penambang berjalan: antarmuka menghentikan
/// penambang lalu node, baru aplikasi keluar. Saat keluar, proses anak yang tersisa dihentikan.
///
/// # Errors
///
/// [`tauri::Error`] jika folder data tidak ditemukan (`Setup`) atau jendela dan WebView gagal dibuat.
#[cfg(not(feature = "vps"))]
pub fn try_run() -> Result<(), tauri::Error> {
    let app = tauri::Builder::default()
        // Tautan dibuka hanya lewat perintah Rust `buka_tautan` (daftar tautan resmi); skrip tautan JS plugin dimatikan dan
        // tanpa capabilities antarmuka tidak bisa memanggil perintah plugin sendiri (pola wallet MOD-15).
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .setup(|app| {
            let folder = mesin::folder_data_bawaan()
                .ok_or("folder data tidak ditemukan (atur ZVRG_DATA)")?;
            let m = Arc::new(mesin::Mesin::baru(
                folder,
                mesin::bin_sidecar("zevrigo-node", "ZEVRIGO_NODE_BIN"),
                mesin::bin_sidecar("zevrigo-miner", "ZEVRIGO_MINER_BIN"),
            ));
            m.mulai_pemantau();
            app.manage(m);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let Some(m) = window
                    .try_state::<Arc<mesin::Mesin>>()
                    .map(|state| Arc::clone(state.inner()))
                else {
                    return;
                };
                if m.ada_proses() && !m.akan_tutup() {
                    api.prevent_close();
                    m.tandai_akan_tutup();
                    let handle = window.app_handle().clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(BATAS_TUTUP);
                        m.hentikan_semua();
                        handle.exit(0);
                    });
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            perintah::mode_aplikasi,
            perintah::info,
            perintah::muat_pengaturan,
            perintah::simpan_pengaturan,
            perintah::nyalakan_node,
            perintah::hentikan_node,
            perintah::keadaan_node,
            perintah::periksa_alamat,
            perintah::mulai_tambang,
            perintah::berhenti_tambang,
            perintah::keadaan_tambang,
            perintah::log_terbaru,
            perintah::buka_folder_data,
            perintah::buka_tautan,
            perintah::akan_tutup,
            perintah::selesai_tutup,
        ])
        .build(tauri::generate_context!())?;
    app.run(|handle, event| {
        if let tauri::RunEvent::Exit = event
            && let Some(m) = handle.try_state::<Arc<mesin::Mesin>>()
        {
            m.hentikan_semua();
        }
    });
    Ok(())
}

/// Menjalankan aplikasi mode VPS (fitur `vps`: Android, juga desktop untuk uji). Node dan penambang berjalan di VPS
/// pengguna; aplikasi hanya mengendalikannya lewat SSH ([`vps::Layanan`]). Brankas di folder data aplikasi
/// (`vps-brankas.json`), atau di `ZVRG_VPS_BRANKAS` bila diatur (uji). Menutup aplikasi tidak menghentikan node.
///
/// # Errors
///
/// [`tauri::Error`] jika folder data aplikasi tidak ditemukan atau jendela dan WebView gagal dibuat.
#[cfg(feature = "vps")]
pub fn try_run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        // Tautan dibuka hanya lewat perintah Rust `buka_tautan` (daftar tautan resmi), seperti desktop.
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .setup(|app| {
            let berkas = match std::env::var_os("ZVRG_VPS_BRANKAS") {
                Some(berkas) => std::path::PathBuf::from(berkas),
                None => app.path().app_data_dir()?.join(vps::BERKAS_BRANKAS),
            };
            app.manage(Arc::new(vps::Layanan::baru(berkas)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            perintah_vps::mode_aplikasi,
            perintah_vps::vps_keadaan,
            perintah_vps::vps_buat_sandi,
            perintah_vps::vps_batal,
            perintah_vps::vps_sambung,
            perintah_vps::vps_pasang,
            perintah_vps::vps_amankan,
            perintah_vps::vps_lewati_amankan,
            perintah_vps::vps_buka,
            perintah_vps::vps_kunci,
            perintah_vps::vps_rpc,
            perintah_vps::vps_status,
            perintah_vps::vps_kendali,
            perintah_vps::vps_ganti_token,
            perintah_vps::vps_ekspor,
            perintah_vps::vps_impor,
            perintah_vps::vps_lupakan,
            perintah_vps::vps_cabut_akses,
            perintah_vps::vps_copot,
            perintah_vps::buka_tautan,
        ])
        .run(tauri::generate_context!())
}
