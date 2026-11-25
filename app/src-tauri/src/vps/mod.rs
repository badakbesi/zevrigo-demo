//! Klien VPS untuk aplikasi Zevrigo Node di Android (RENCANA_T17 bagian 9 dan 13, DEC-45, DEC-46; Q199a).
//!
//! Aplikasi tersambung ke VPS milik pengguna lewat SSH (russh + kripto ring) dan hanya menjalankan:
//! - skrip pemeriksaan dan pemasangan tetap di modul ini ([`periksa_vps`], [`pasang_baru`], [`pasang_kunci_publik`]);
//! - skrip bantu paket VPS `demo-zevrigo-kendali` dengan perintah tetap ([`Kendali`]);
//! - JSON-RPC pengendali `zevrigo-demo pengendali` di 127.0.0.1:28808 lewat terowongan SSH ([`Sesi::rpc`]).
//!
//! Tidak ada perintah bebas dari antarmuka. Sidik kunci host dipercaya pada sambungan pertama, lalu wajib sama
//! ([`Galat::SidikBerubah`]). Paket VPS yang boleh dipasang tertanam di aplikasi ([`PAKET_SHA256`], pencipta 25 November 2026).
//! Modul ini hanya ada dengan fitur `vps` (build Android dan uji `examples/uji-vps.rs`); program desktop tidak memuatnya.

mod kendali;
mod pasang;
mod rpc;
mod sesi;
#[cfg(test)]
mod tests;

pub use kendali::{Kendali, SKRIP_BANTU, StatusVps};
pub use pasang::{
    Kunci, PAKET, PAKET_SHA256, PemeriksaanVps, RILIS, buat_token, pasang_baru,
    pasang_kunci_publik, periksa_vps, sha256_hex,
};
pub use rpc::METODE;
pub use sesi::{Galat, Keluaran, Masuk, Sasaran, Sesi};
