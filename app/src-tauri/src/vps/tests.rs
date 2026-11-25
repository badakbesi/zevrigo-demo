//! Uji unit modul `vps` (tanpa jaringan). Uji sungguhan ke VPS: `examples/uji-vps.rs` (Q199a).

use std::net::IpAddr;

use serde_json::{Value, json};

use super::kendali::blob_kunci_sah;
use super::pasang::{BERKAS_PASANG, hex64};
use super::rpc::{hasil_rpc, permintaan_http, token_sah, urai_http};
use super::sesi::nama_pengguna_sah;
use super::{
    Galat, Keluaran, Kendali, Kunci, METODE, PAKET, PAKET_SHA256, PemeriksaanVps, RILIS, Sasaran,
    StatusVps, buat_token, sha256_hex,
};

#[test]
fn user_and_target_checks() {
    assert!(nama_pengguna_sah("root"));
    assert!(nama_pengguna_sah("zvrg"));
    assert!(nama_pengguna_sah("demo-zevrigo-uji"));
    assert!(nama_pengguna_sah("_a1"));
    assert!(!nama_pengguna_sah(""));
    assert!(!nama_pengguna_sah("1abc"));
    assert!(!nama_pengguna_sah("Root"));
    assert!(!nama_pengguna_sah("a b"));
    assert!(!nama_pengguna_sah("a;rm"));
    assert!(!nama_pengguna_sah(&"a".repeat(33)));
    assert!(nama_pengguna_sah(&"a".repeat(32)));

    let s = Sasaran::baru(" 103.67.79.110 ", 20121, "zvrg").unwrap();
    assert_eq!(s.host(), "103.67.79.110");
    assert_eq!(s.port(), 20121);
    assert_eq!(s.pengguna(), "zvrg");
    assert!(Sasaran::baru("vps.contoh.id", 22, "root").is_ok());
    assert!(Sasaran::baru("2001:db8::1", 22, "root").is_ok());
    assert!(matches!(
        Sasaran::baru("", 22, "root"),
        Err(Galat::Masukan(_))
    ));
    assert!(matches!(
        Sasaran::baru("a b", 22, "root"),
        Err(Galat::Masukan(_))
    ));
    assert!(matches!(
        Sasaran::baru("vps;id", 22, "root"),
        Err(Galat::Masukan(_))
    ));
    assert!(matches!(
        Sasaran::baru("vps", 0, "root"),
        Err(Galat::Masukan(_))
    ));
    assert!(matches!(
        Sasaran::baru("vps", 22, "x y"),
        Err(Galat::Masukan(_))
    ));
}

#[test]
fn helper_commands_are_fixed_and_checked() {
    let sha = "ab".repeat(32);
    assert_eq!(
        Kendali::Status.baris(false).unwrap(),
        "sudo -n /usr/local/sbin/demo-zevrigo-kendali status"
    );
    assert_eq!(
        Kendali::Status.baris(true).unwrap(),
        "/usr/local/sbin/demo-zevrigo-kendali status"
    );
    assert_eq!(
        Kendali::Perbarui.baris(true).unwrap(),
        format!("/usr/local/sbin/demo-zevrigo-kendali perbarui {PAKET_SHA256}")
    );
    assert!(
        Kendali::Token(sha.clone())
            .baris(false)
            .unwrap()
            .ends_with(&format!(" token {sha}"))
    );
    assert!(matches!(
        Kendali::Token("AB".repeat(32)).baris(false),
        Err(Galat::Masukan(_))
    ));
    assert!(matches!(
        Kendali::Token("ab; reboot".to_owned()).baris(false),
        Err(Galat::Masukan(_))
    ));
    let v4: IpAddr = "203.0.113.7".parse().unwrap();
    assert!(
        Kendali::NodePublikNyala {
            ip: v4,
            port: 28800
        }
        .baris(true)
        .unwrap()
        .ends_with(" node-publik nyala 203.0.113.7 28800")
    );
    let v6: IpAddr = "2001:db8::7".parse().unwrap();
    assert!(
        Kendali::NodePublikNyala {
            ip: v6,
            port: 30000
        }
        .baris(true)
        .unwrap()
        .ends_with(" node-publik nyala 2001:db8::7 30000")
    );
    let campuran: IpAddr = "::ffff:203.0.113.7".parse().unwrap();
    assert!(
        Kendali::NodePublikNyala {
            ip: campuran,
            port: 28800
        }
        .baris(true)
        .is_err()
    );
    assert!(
        Kendali::NodePublikNyala { ip: v4, port: 80 }
            .baris(true)
            .is_err()
    );
    assert!(
        Kendali::NodePublikMati
            .baris(true)
            .unwrap()
            .ends_with(" node-publik mati")
    );
    assert!(
        Kendali::BatasCpu(50)
            .baris(true)
            .unwrap()
            .ends_with(" batas-cpu 50")
    );
    assert!(Kendali::BatasCpu(9).baris(true).is_err());
    assert!(Kendali::BatasCpu(101).baris(true).is_err());
    assert!(
        Kendali::AmankanSsh
            .baris(true)
            .unwrap()
            .ends_with(" amankan-ssh")
    );
    assert!(
        Kendali::RestartVps
            .baris(true)
            .unwrap()
            .ends_with(" restart-vps")
    );
    assert!(
        Kendali::Copot { hapus_data: true }
            .baris(true)
            .unwrap()
            .ends_with(" copot --hapus-data")
    );
    assert!(
        Kendali::Copot { hapus_data: false }
            .baris(true)
            .unwrap()
            .ends_with(" copot")
    );
    let blob = format!("AAAAC3NzaC1lZDI1NTE5AAAA{}", "x".repeat(44));
    assert!(blob_kunci_sah(&blob));
    assert!(
        Kendali::CabutKunci(blob.clone())
            .baris(false)
            .unwrap()
            .ends_with(&format!(" cabut-kunci {blob}"))
    );
    assert!(!blob_kunci_sah("pendek"));
    assert!(!blob_kunci_sah(&format!("{blob} x")));
    assert!(!blob_kunci_sah(&format!("{blob}===")));
    assert!(
        Kendali::CabutKunci(format!("{blob};id"))
            .baris(true)
            .is_err()
    );
}

#[test]
fn reads_helper_status() {
    let status: StatusVps = serde_json::from_value(json!({
        "versi": "demo-zevrigo-vps zevrigo-demo faa9c3a inti be71d57 2026-11-25",
        "layanan": "active",
        "aktifSejak": "Wed 2026-11-25 09:39:50 UTC",
        "cpu": 1,
        "muatan1m": "0.34",
        "ramTotalMb": 3911,
        "ramTersediaMb": 3218,
        "diskTotalMb": 18687,
        "diskBebasMb": 13100,
        "uptimeDetik": 691_618,
        "nodePublik": "",
        "kuotaCpu": "500ms",
        "loginPassword": "no"
    }))
    .unwrap();
    assert_eq!(status.layanan, "active");
    assert_eq!(status.kuota_cpu, "500ms");
    assert_eq!(status.disk_bebas_mb, 13100);
}

#[test]
fn reads_controller_http_answers() {
    // Badan lewat json! (bukan raw string): pemindai larangan membaca angka versi di dalam raw string sebagai pecahan.
    let badan =
        serde_json::to_vec(&json!({"id": 1, "jsonrpc": "2.0", "result": {"versi": "0.1.0"}}))
            .unwrap();
    let mut jawaban = format!(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n",
        badan.len()
    )
    .into_bytes();
    jawaban.extend_from_slice(&badan);
    let (status, isi) = urai_http(&jawaban).unwrap();
    assert_eq!(status, 200);
    assert_eq!(isi, badan);
    assert_eq!(hasil_rpc(&jawaban).unwrap(), json!({"versi": "0.1.0"}));

    let chunked = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n";
    assert_eq!(urai_http(chunked).unwrap().1, b"hello world");

    assert_eq!(
        hasil_rpc(b"HTTP/1.1 401 Unauthorized\r\ncontent-length: 0\r\n\r\n"),
        Err(Galat::TokenDitolak)
    );
    assert!(matches!(
        hasil_rpc(b"HTTP/1.1 429 Too Many Requests\r\ncontent-length: 0\r\n\r\n"),
        Err(Galat::Pengendali(_))
    ));
    let galat = serde_json::to_vec(&json!({"jsonrpc": "2.0", "id": 1, "error": {"code": -32601, "message": "metode tidak dikenal"}}))
        .unwrap();
    let mut jawaban =
        format!("HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n", galat.len()).into_bytes();
    jawaban.extend_from_slice(&galat);
    assert_eq!(
        hasil_rpc(&jawaban),
        Err(Galat::Pengendali(
            "metode tidak dikenal (kode -32601)".to_owned()
        ))
    );
    let kosong = serde_json::to_vec(&json!({"jsonrpc": "2.0", "id": 1, "result": null})).unwrap();
    let mut jawaban = format!(
        "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n",
        kosong.len()
    )
    .into_bytes();
    jawaban.extend_from_slice(&kosong);
    assert_eq!(hasil_rpc(&jawaban).unwrap(), Value::Null);
    assert!(urai_http(b"bukan http").is_err());
    assert!(urai_http(b"HTTP/1.1 200 OK\r\ncontent-length: 10\r\n\r\npendek").is_err());

    let permintaan = String::from_utf8(permintaan_http(&"a".repeat(64), b"{}")).unwrap();
    assert!(permintaan.starts_with("POST / HTTP/1.1\r\n"));
    assert!(permintaan.contains(&format!("\r\nAuthorization: Bearer {}\r\n", "a".repeat(64))));
    assert!(permintaan.contains("\r\nContent-Length: 2\r\n"));
    assert!(permintaan.contains("\r\nConnection: close\r\n"));
    assert!(permintaan.ends_with("\r\n\r\n{}"));
}

#[test]
fn tokens_methods_and_hashes() {
    let a = buat_token().unwrap();
    let b = buat_token().unwrap();
    assert_eq!(a.len(), 64);
    assert!(a.chars().all(|c| c.is_ascii_alphanumeric()));
    assert_ne!(*a, *b);
    assert!(token_sah(&a));
    assert!(!token_sah(&"a".repeat(31)));
    assert!(!token_sah(&"a".repeat(257)));
    assert!(!token_sah(&format!("{}\r\nX: y", "a".repeat(40))));
    assert_eq!(
        sha256_hex("abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert!(hex64(&sha256_hex(&a)));
    assert!(hex64(PAKET_SHA256));
    assert!(METODE.contains(&"saldo"));
    assert!(!METODE.contains(&"jalankan"));
}

#[test]
fn app_key_round_trip() {
    let kunci = Kunci::baru().unwrap();
    let baris = kunci.baris_publik().unwrap();
    assert!(baris.starts_with("ssh-ed25519 "));
    assert!(baris.ends_with(" zevrigo-node-android"));
    let blob = kunci.blob_publik().unwrap();
    assert!(blob_kunci_sah(&blob));
    assert_eq!(baris, format!("ssh-ed25519 {blob} zevrigo-node-android"));
    let teks = kunci.ke_openssh().unwrap();
    assert!(teks.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----"));
    let lagi = Kunci::dari_openssh(&teks).unwrap();
    assert_eq!(lagi.baris_publik().unwrap(), baris);
    assert_ne!(Kunci::baru().unwrap().baris_publik().unwrap(), baris);
    assert!(Kunci::dari_openssh("bukan kunci").is_err());
}

#[test]
fn reads_vps_check() {
    let teks = [
        "arsitektur=x86_64",
        "os=ubuntu",
        "versi_os=26.04",
        "nama_os=Ubuntu 26.04.1 LTS",
        "systemd=ya",
        "sudo=ya",
        "unduh=ya",
        "ca=ya",
        "disk_bebas_mb=13176",
        "ram_mb=3911",
    ]
    .join("\n");
    let p = PemeriksaanVps::dari_teks(&teks);
    assert_eq!(p.os, "ubuntu");
    assert_eq!(p.nama_os, "Ubuntu 26.04.1 LTS");
    assert!(p.systemd && p.sudo && !p.root && !p.terpasang);
    assert_eq!(p.disk_bebas_mb, 13176);
    assert!(p.masalah().is_empty(), "{:?}", p.masalah());

    let lama = PemeriksaanVps::dari_teks(
        "arsitektur=aarch64\nos=ubuntu\nversi_os=20.04\nunduh=ya\nca=ya\n",
    );
    let m = lama.masalah();
    assert!(m.iter().any(|x| x.contains("x86_64")));
    assert!(m.iter().any(|x| x.contains("belum didukung")));
    assert!(m.iter().any(|x| x.contains("systemd")));
    assert!(m.iter().any(|x| x.contains("root atau")));
    let debian = PemeriksaanVps::dari_teks(
        "arsitektur=x86_64\nos=debian\nversi_os=12\nsystemd=ya\nroot=ya\nunduh=ya\nca=ya\ndisk_bebas_mb=5000\nram_mb=960\n",
    );
    assert!(debian.masalah().is_empty(), "{:?}", debian.masalah());
}

#[test]
fn scripts_match_constants() {
    let pasang = include_str!("pasang.sh");
    assert!(pasang.contains(&format!("RILIS={RILIS}\n")));
    assert!(pasang.contains(&format!("PAKET={PAKET}\n")));
    assert!(pasang.contains("demo-zevrigo-kendali\" pasang --pengguna-ssh"));
    assert!(!pasang.contains("| grep -q"));
    assert!(BERKAS_PASANG.starts_with(".cache/"));
    assert!(!include_str!("periksa.sh").contains("sudo -"));
}

#[test]
fn command_output_messages() {
    let k = Keluaran {
        kode: Some(1),
        stdout: Vec::new(),
        stderr: b"peringatan lain\ngalat: SHA-256 paket yang diunduh tidak cocok\n\n".to_vec(),
    };
    assert!(!k.berhasil());
    assert_eq!(k.pesan_galat(), "SHA-256 paket yang diunduh tidak cocok");
    assert_eq!(
        k.wajib_berhasil(),
        Err(Galat::Perintah {
            kode: Some(1),
            pesan: "SHA-256 paket yang diunduh tidak cocok".to_owned()
        })
    );
    let sudo = Keluaran {
        kode: Some(1),
        stdout: Vec::new(),
        stderr: b"sudo: 1 incorrect password attempt\n".to_vec(),
    };
    assert_eq!(sudo.pesan_galat(), "sudo: 1 incorrect password attempt");
    let ok = Keluaran {
        kode: Some(0),
        stdout: b"terpasang: rilis-1\n".to_vec(),
        stderr: Vec::new(),
    };
    assert_eq!(ok.wajib_berhasil().unwrap().teks(), "terpasang: rilis-1");
}
