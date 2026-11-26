//! Uji unit layanan VPS dan brankas (Q199b, tanpa jaringan). Uji sungguhan ke VPS: `examples/uji-layanan.rs`.

use std::path::PathBuf;

use serde_json::json;
use zevrigo_wallet_core::vault::KdfParams;

use super::aplikasi::{Layanan, PerintahUi, Tahap};
use super::brankas::{Rahasia, buka_brankas, sandi_sah, tutup_brankas};
use super::{Galat, Kendali, Kunci};

/// Argon2id ringan untuk uji (build debug); aplikasi memakai bawaan wallet.
const RINGAN: KdfParams = KdfParams {
    memory_kib: 8,
    iterations: 1,
    parallelism: 1,
};

fn contoh(amankan: bool) -> Rahasia {
    Rahasia {
        host: "203.0.113.7".to_owned(),
        port: 22,
        pengguna: "zvrg".to_owned(),
        sidik: "SHA256:uji".to_owned(),
        kunci: Kunci::baru()
            .unwrap()
            .ke_openssh()
            .unwrap()
            .as_str()
            .to_owned(),
        token: "a".repeat(64),
        amankan_selesai: amankan,
    }
}

fn folder_uji(nama: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("zvrg-q199b-{}-{nama}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    folder
}

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn vault_round_trip_and_errors() {
    let r = contoh(true);
    let teks = tutup_brankas(&r, "sandi-aplikasi", RINGAN).unwrap();
    assert!(!teks.contains(&r.token));
    assert!(!teks.contains("OPENSSH"));
    assert_eq!(buka_brankas(&teks, "sandi-aplikasi").unwrap(), r);
    assert_eq!(
        buka_brankas(&teks, "sandi-lain-salah"),
        Err(Galat::SandiSalah)
    );
    assert_ne!(tutup_brankas(&r, "sandi-aplikasi", RINGAN).unwrap(), teks);

    let mut v: serde_json::Value = serde_json::from_str(&teks).unwrap();
    let isi = v["isi"].as_str().unwrap().to_owned();
    let akhir = if isi.ends_with('0') { '1' } else { '0' };
    v["isi"] = json!(format!("{}{akhir}", &isi[..isi.len() - 1]));
    assert_eq!(
        buka_brankas(&v.to_string(), "sandi-aplikasi"),
        Err(Galat::SandiSalah)
    );
    v["jenis"] = json!("lain");
    assert!(matches!(
        buka_brankas(&v.to_string(), "sandi-aplikasi"),
        Err(Galat::Brankas(_))
    ));
    assert!(matches!(
        buka_brankas("bukan data", "sandi-aplikasi"),
        Err(Galat::Brankas(_))
    ));
    assert!(matches!(
        tutup_brankas(&r, "pendek", RINGAN),
        Err(Galat::Masukan(_))
    ));

    assert!(!sandi_sah("1234567"));
    assert!(sandi_sah("12345678"));
    assert!(!sandi_sah(&"\u{e9}".repeat(7)));
    assert!(sandi_sah(&"\u{e9}".repeat(8)));
    assert!(sandi_sah(&"x".repeat(256)));
    assert!(!sandi_sah(&"x".repeat(257)));
    let debug = format!("{r:?}");
    assert!(!debug.contains(&r.token) && !debug.contains("OPENSSH"));
}

#[test]
fn service_stages_without_network() {
    let rt = rt();
    let berkas = folder_uji("tahap").join("vps-brankas.json");
    let l = Layanan::dengan_kdf(berkas.clone(), RINGAN);
    assert_eq!(l.info().unwrap().tahap, Tahap::Baru);
    assert!(matches!(l.buat_sandi("pendek"), Err(Galat::Masukan(_))));
    assert_eq!(
        l.buat_sandi("sandi-aplikasi").unwrap().tahap,
        Tahap::Sambung
    );
    assert!(matches!(
        rt.block_on(l.sambung_awal("vps;id", 22, "root", "x")),
        Err(Galat::Masukan(_))
    ));
    assert!(matches!(
        rt.block_on(l.sambung_awal("203.0.113.7", 22, "root", "")),
        Err(Galat::Masukan(_))
    ));
    assert!(matches!(rt.block_on(l.pasang()), Err(Galat::Tahap(_))));
    assert_eq!(rt.block_on(l.batal()).unwrap().tahap, Tahap::Baru);
    assert!(matches!(
        rt.block_on(l.sambung_awal("203.0.113.7", 22, "root", "x")),
        Err(Galat::Tahap(_))
    ));
    assert_eq!(rt.block_on(l.rpc("info", json!({}))), Err(Galat::Terkunci));
    assert!(matches!(rt.block_on(l.status()), Err(Galat::Terkunci)));
    assert!(matches!(l.ekspor(), Err(Galat::Terkunci)));
    assert!(matches!(l.lewati_amankan(), Err(Galat::Terkunci)));
    assert!(matches!(rt.block_on(l.cabut_akses()), Err(Galat::Terkunci)));

    let teks = tutup_brankas(&contoh(false), "sandi-aplikasi", RINGAN).unwrap();
    assert_eq!(l.impor(&teks, "sandi-lain-salah"), Err(Galat::SandiSalah));
    assert!(!berkas.exists());
    let info = l.impor(&teks, "sandi-aplikasi").unwrap();
    assert_eq!(info.tahap, Tahap::Amankan);
    assert_eq!(info.host.as_deref(), Some("203.0.113.7"));
    assert_eq!(info.sidik.as_deref(), Some("SHA256:uji"));
    assert_eq!(l.ekspor().unwrap(), teks.trim());
    assert!(matches!(
        l.impor(&teks, "sandi-aplikasi"),
        Err(Galat::Tahap(_))
    ));
    assert!(matches!(
        l.buat_sandi("sandi-aplikasi"),
        Err(Galat::Tahap(_))
    ));
    assert_eq!(l.lewati_amankan().unwrap().tahap, Tahap::Siap);
    assert_eq!(rt.block_on(l.kunci()).unwrap().tahap, Tahap::Terkunci);
    assert_eq!(l.buka("sandi-lain-salah"), Err(Galat::SandiSalah));
    assert_eq!(l.buka("sandi-aplikasi").unwrap().tahap, Tahap::Amankan);
    assert_eq!(rt.block_on(l.lupakan()).unwrap().tahap, Tahap::Baru);
    assert!(!berkas.exists());

    let siap = tutup_brankas(&contoh(true), "sandi-aplikasi", RINGAN).unwrap();
    assert_eq!(l.impor(&siap, "sandi-aplikasi").unwrap().tahap, Tahap::Siap);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(&berkas).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    assert_eq!(rt.block_on(l.lupakan()).unwrap().tahap, Tahap::Baru);
    let _ = std::fs::remove_dir_all(berkas.parent().unwrap());
}

#[test]
fn ui_commands_map_to_helper_commands() {
    let ubah = |v: serde_json::Value| serde_json::from_value::<PerintahUi>(v);
    assert_eq!(
        ubah(json!({"jenis": "batas_cpu", "persen": 30}))
            .unwrap()
            .ke_kendali(),
        Ok(Kendali::BatasCpu(30))
    );
    assert_eq!(
        ubah(json!({"jenis": "node_publik_nyala", "ip": " 203.0.113.7 ", "port": 28800}))
            .unwrap()
            .ke_kendali(),
        Ok(Kendali::NodePublikNyala {
            ip: "203.0.113.7".parse().unwrap(),
            port: 28800
        })
    );
    assert!(matches!(
        ubah(json!({"jenis": "node_publik_nyala", "ip": "bukan-ip", "port": 28800}))
            .unwrap()
            .ke_kendali(),
        Err(Galat::Masukan(_))
    ));
    assert_eq!(
        ubah(json!({"jenis": "perbarui"})).unwrap().ke_kendali(),
        Ok(Kendali::Perbarui)
    );
    assert_eq!(
        ubah(json!({"jenis": "node_publik_mati"}))
            .unwrap()
            .ke_kendali(),
        Ok(Kendali::NodePublikMati)
    );
    assert_eq!(
        ubah(json!({"jenis": "restart_vps"})).unwrap().ke_kendali(),
        Ok(Kendali::RestartVps)
    );
    for jenis in [
        "copot",
        "amankan_ssh",
        "token",
        "cabut_kunci",
        "status",
        "jalankan",
    ] {
        assert!(ubah(json!({ "jenis": jenis })).is_err(), "{jenis}");
    }
    assert!(ubah(json!({"jenis": "batas_cpu", "persen": 300})).is_err());
}

#[test]
fn error_codes() {
    assert_eq!(Galat::SandiSalah.kode(), "sandi_salah");
    assert_eq!(Galat::Terkunci.kode(), "terkunci");
    assert_eq!(
        Galat::SidikBerubah {
            dikenal: "a".to_owned(),
            terlihat: "b".to_owned()
        }
        .kode(),
        "sidik_berubah"
    );
    assert_eq!(Galat::Tahap("x").kode(), "tahap");
    assert!(Galat::Sambung("x".to_owned()).putus());
    assert!(Galat::BatasWaktu("x").putus());
    assert!(!Galat::TokenDitolak.putus());
    assert!(!Galat::SandiSalah.putus());
    assert_eq!(Galat::SandiSalah.to_string(), "kata sandi aplikasi salah");
}
