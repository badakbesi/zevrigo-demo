use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use zevrigo_primitives::{Network, OutputKind};
use zevrigo_wallet_core::encode_address;

use super::{
    BarisLog, JEDA_SAH, MAKS_LOG, Mesin, PenampungLog, Pengaturan, argumen_node, bersihkan_baris,
    blok_dari_baris, periksa_alamat, rekan_dari_baris, status_node, tautan_resmi, tingkat_baris,
    urai_publik,
};

#[test]
fn address_checks_network_and_gives_payload() {
    let payload = [7_u8; 32];
    let sah = encode_address(Network::Regtest, OutputKind::SingleKey, &payload);
    let (hasil, hadiah) = periksa_alamat(&format!("  {sah} "));
    assert!(hasil.sah);
    assert_eq!(hasil.alasan, None);
    let (jenis, hex) = hadiah.unwrap();
    assert_eq!(jenis, 0);
    assert_eq!(hex, "07".repeat(32));

    let testnet = encode_address(Network::Testnet, OutputKind::SingleKey, &payload);
    assert_eq!(periksa_alamat(&testnet).0.alasan, Some("jaringan-lain"));
    assert!(periksa_alamat(&testnet).1.is_none());
    assert_eq!(periksa_alamat("").0.alasan, Some("kosong"));
    assert_eq!(periksa_alamat("   ").0.alasan, Some("kosong"));
    let mut rusak = sah.clone();
    rusak.pop();
    rusak.push(if sah.ends_with('q') { 'p' } else { 'q' });
    assert_eq!(periksa_alamat(&rusak).0.alasan, Some("format"));
    assert_eq!(periksa_alamat("bukan alamat").0.alasan, Some("format"));
}

#[test]
fn reads_peer_and_block_lines() {
    assert_eq!(
        rekan_dari_baris("2026-11-24T01:00:00Z  INFO rekan terhubung peer=12D3 address=/dns4/x"),
        Some(1)
    );
    assert_eq!(
        rekan_dari_baris("2026-11-24T01:00:00Z  INFO rekan terputus peer=12D3"),
        Some(-1)
    );
    assert_eq!(rekan_dari_baris("P2P mendengar"), None);

    let id = "0084cb380e8023bc6ebb6180c5cae0f2706f45c129d3cfdb1b45980c9b629e6a";
    assert_eq!(
        blok_dari_baris(&format!("blok diterima: tinggi 2001, nonce 1, id {id}")),
        Some((2001, id.to_owned()))
    );
    assert_eq!(blok_dari_baris("selesai: 1 blok ditambang"), None);
    assert_eq!(
        blok_dari_baris("blok diterima: tinggi x, nonce 1, id 00"),
        None
    );
    assert_eq!(
        blok_dari_baris("blok diterima: tinggi 5, nonce 1, id 00ff"),
        None
    );

    assert_eq!(tingkat_baris("2026 WARN rekan disanksi"), "peringatan");
    assert_eq!(tingkat_baris("2026 ERROR penyimpanan gagal"), "galat");
    assert_eq!(tingkat_baris("2026  INFO dial gagal, dicoba lagi"), "info");
    assert_eq!(
        tingkat_baris("zevrigo-miner: RPC gagal: koneksi ditolak"),
        "galat"
    );
    assert_eq!(tingkat_baris("blok diterima: tinggi 5"), "info");
    assert_eq!(
        bersihkan_baris("\u{1b}[2m2026\u{1b}[0m \u{1b}[32m INFO\u{1b}[0m rekan\tterhubung\r"),
        "2026  INFO rekan terhubung"
    );
    assert_eq!(bersihkan_baris("tanpa warna"), "tanpa warna");
}

#[test]
fn only_official_links_open() {
    for url in [
        "https://zevrigo.online/",
        "https://zevrigo.online/unduh/",
        "https://demo-explorer.zevrigo.online",
        "https://demo-explorer.zevrigo.online/block/?id=2001",
        "https://github.com/badakbesi/zevrigo-demo/issues",
    ] {
        assert!(tautan_resmi(url), "{url}");
    }
    for url in [
        "http://zevrigo.online/",
        "https://zevrigo.online.contoh.test/",
        "https://demo-explorer.zevrigo.online.contoh.test/",
        "https://github.com/badakbesi/zevrigo-demo/issues/../../lain",
        "file:///etc/passwd",
        "",
    ] {
        assert!(!tautan_resmi(url), "{url}");
    }
}

#[test]
fn node_status_follows_observations() {
    assert_eq!(status_node(false, false, false, None, None, 0), "mati");
    assert_eq!(
        status_node(true, false, false, None, Some(10), 0),
        "menyala"
    );
    assert_eq!(
        status_node(true, false, false, Some(0), Some(10), 0),
        "menyambung"
    );
    assert_eq!(
        status_node(true, false, false, Some(5), Some(10), 1),
        "mengejar"
    );
    assert_eq!(
        status_node(true, false, false, Some(10), Some(10), 1),
        "sinkron"
    );
    assert_eq!(
        status_node(true, false, false, Some(11), Some(10), 2),
        "sinkron"
    );
    assert_eq!(
        status_node(true, false, false, Some(3), None, 1),
        "mengejar"
    );
    assert_eq!(
        status_node(true, false, false, Some(0), None, 1),
        "mengejar"
    );
    assert_eq!(
        status_node(false, true, false, Some(10), None, 0),
        "mengejar"
    );
    assert_eq!(
        status_node(false, true, false, Some(10), Some(10), 0),
        "sinkron"
    );
    assert_eq!(status_node(false, true, false, None, None, 0), "menyambung");
    assert_eq!(status_node(false, false, true, None, None, 0), "galat");
}

#[test]
fn node_arguments_and_public_mode() {
    let folder = PathBuf::from("/var/lib/demo-zevrigo");
    let dasar = [
        "--regtest",
        "--datadir",
        "/var/lib/demo-zevrigo",
        "--seed-resmi",
    ];
    let teks = |a: Vec<std::ffi::OsString>| -> Vec<String> {
        a.into_iter().map(|x| x.into_string().unwrap()).collect()
    };
    assert_eq!(teks(argumen_node(&folder, None)), dasar);
    assert_eq!(
        teks(argumen_node(&folder, Some("/ip4/203.0.113.7/tcp/28800"))),
        [
            &dasar[..],
            &[
                "--p2p-bind",
                "0.0.0.0",
                "--p2p-port",
                "28800",
                "--p2p-publik",
                "/ip4/203.0.113.7/tcp/28800"
            ],
        ]
        .concat()
    );
    assert_eq!(
        urai_publik("/ip6/2001:db8::7/tcp/30000"),
        Some(("::", 30000))
    );
    for salah in [
        "",
        "/ip4/203.0.113.7/tcp/80",
        "/ip4/203.0.113.7/tcp/70000",
        "/ip4/203.0.113/tcp/28800",
        "/ip4/203.0.113.7/udp/28800",
        "/ip4/203.0.113.7/tcp/28800/p2p/12D3",
        "/dns4/contoh.test/tcp/28800",
        "/ip6/203.0.113.7/tcp/28800",
    ] {
        assert_eq!(urai_publik(salah), None, "{salah}");
        assert_eq!(teks(argumen_node(&folder, Some(salah))), dasar, "{salah}");
    }
}

#[test]
fn log_keeps_increasing_time_and_limit() {
    let mut log = PenampungLog::default();
    for i in 0..(MAKS_LOG + 20) {
        log.tambah("node", "info", &format!("baris {i}"));
    }
    let semua: Vec<BarisLog> = log.sejak(None);
    assert_eq!(semua.len(), MAKS_LOG);
    assert!(semua.windows(2).all(|w| w[0].waktu < w[1].waktu));
    assert_eq!(
        semua.last().unwrap().teks,
        format!("baris {}", MAKS_LOG + 19)
    );
    let tengah = semua[MAKS_LOG - 3].waktu;
    assert_eq!(log.sejak(Some(tengah)).len(), 2);
    log.tambah("penambang", "info", "blok diterima");
    log.tambah("node", "info", "rekan terhubung");
    assert_eq!(log.ekor(1, "penambang"), "blok diterima");
    assert_eq!(
        log.ekor(2, "node"),
        format!("baris {}\nrekan terhubung", MAKS_LOG + 19)
    );
}

#[test]
fn settings_are_normalized_and_saved() {
    let p = Pengaturan::dari_json(
        r#"{"alamatHadiah": " rzvrg1abc ", "jedaDetik": 61, "tema": "ungu"}"#,
    );
    assert_eq!(p.alamat_hadiah, "rzvrg1abc");
    assert_eq!(p.jeda_detik, 120);
    assert_eq!(p.tema, "sistem");
    assert_eq!(Pengaturan::dari_json("bukan json"), Pengaturan::default());
    assert_eq!(
        Pengaturan::dari_json(r#"{"jedaDetik": 300}"#).jeda_detik,
        300
    );
    for jeda in JEDA_SAH {
        let p = Pengaturan::dari_json(&format!(r#"{{"jedaDetik": {jeda}, "tema": "gelap"}}"#));
        assert_eq!((p.jeda_detik, p.tema.as_str()), (jeda, "gelap"));
    }

    let dir = std::env::temp_dir().join(format!("zvrg-node-app-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mesin = Mesin::baru(
        dir.clone(),
        PathBuf::from("tidak-ada"),
        PathBuf::from("tidak-ada"),
    );
    assert_eq!(mesin.muat_pengaturan(), Pengaturan::default());
    let simpan = Pengaturan {
        alamat_hadiah: "rzvrg1q".to_owned(),
        jeda_detik: 60,
        tema: "terang".to_owned(),
    };
    mesin.simpan_pengaturan(simpan.clone()).unwrap();
    assert_eq!(mesin.muat_pengaturan(), simpan);
    let json = std::fs::read_to_string(dir.join(super::BERKAS_PENGATURAN)).unwrap();
    assert!(json.contains("\"alamatHadiah\""), "{json}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn refuses_mining_before_sync_and_missing_node() {
    let dir = std::env::temp_dir().join(format!("zvrg-node-app-tolak-{}", std::process::id()));
    let mesin = Arc::new(Mesin::baru(
        dir.clone(),
        PathBuf::from("tidak-ada-node"),
        PathBuf::from("tidak-ada-miner"),
    ));
    let alamat = encode_address(Network::Regtest, OutputKind::SingleKey, &[1; 32]);
    assert!(mesin.mulai_tambang(&alamat, 120).is_err());
    assert_eq!(mesin.keadaan_tambang().status, "berhenti");
    assert!(
        mesin
            .keadaan_tambang()
            .pesan_galat
            .unwrap()
            .contains("belum sinkron")
    );
    assert!(mesin.mulai_tambang(&alamat, 90).is_err());
    assert!(mesin.mulai_tambang("rzvrg1salah", 120).is_err());
    assert_eq!(mesin.keadaan_node().status, "mati");
    assert!(!mesin.ada_proses());
    let _ = std::fs::remove_dir_all(&dir);
}

/// Uji sungguhan (Q194a J4, dijalankan skrip dengan `--ignored`): node dari `ZEVRIGO_NODE_BIN` tersambung ke seed jaringan
/// demo, sinkron, penambang dari `ZEVRIGO_MINER_BIN` menambang satu blok, lalu semua dihentikan dengan rapi. Butuh internet
/// dan port 28800/28801 bebas; folder data dari `ZVRG_DATA`.
#[test]
#[ignore = "butuh internet, jaringan demo, dan sidecar sungguhan"]
fn real_node_syncs_mines_and_stops() {
    let folder = PathBuf::from(std::env::var_os("ZVRG_DATA").unwrap());
    let mesin = Arc::new(Mesin::baru(
        folder,
        super::bin_sidecar("zevrigo-node", "ZEVRIGO_NODE_BIN"),
        super::bin_sidecar("zevrigo-miner", "ZEVRIGO_MINER_BIN"),
    ));
    mesin.mulai_pemantau();
    assert!(
        mesin.versi_node().starts_with("zevrigo-node"),
        "{}",
        mesin.versi_node()
    );
    mesin.nyalakan_node().unwrap();
    assert!(
        !mesin.node_luar.load(std::sync::atomic::Ordering::SeqCst),
        "node lain sudah berjalan"
    );
    let tunggu = |batas: Duration, mut syarat: Box<dyn FnMut() -> bool>| {
        let akhir = Instant::now() + batas;
        while Instant::now() < akhir {
            if syarat() {
                return true;
            }
            std::thread::sleep(Duration::from_secs(2));
        }
        false
    };
    let m = Arc::clone(&mesin);
    let sinkron = tunggu(
        Duration::from_secs(600),
        Box::new(move || m.keadaan_node().status == "sinkron"),
    );
    let node = mesin.keadaan_node();
    println!("node: {node:?}");
    assert!(sinkron, "node tidak sinkron dalam 10 menit: {node:?}");
    assert!(node.rekan >= 1);
    let jaringan = node.tinggi_jaringan.unwrap();
    assert!(
        jaringan > 0 && node.tinggi_lokal.unwrap() >= jaringan,
        "{node:?}"
    );

    let alamat = encode_address(Network::Regtest, OutputKind::SingleKey, &[0x5a; 32]);
    mesin.mulai_tambang(&alamat, 60).unwrap();
    let m = Arc::clone(&mesin);
    let blok = tunggu(
        Duration::from_secs(300),
        Box::new(move || m.keadaan_tambang().blok_sesi_ini >= 1),
    );
    let tambang = mesin.keadaan_tambang();
    println!("tambang: {tambang:?}");
    assert!(blok, "tidak ada blok dalam 5 menit: {tambang:?}");
    assert_eq!(tambang.blok_terakhir[0].id_blok.len(), 64);

    mesin.hentikan_semua();
    assert_eq!(mesin.keadaan_tambang().status, "berhenti");
    assert_eq!(mesin.keadaan_node().status, "mati");
    assert!(!mesin.ada_proses());
    let log: Vec<String> = mesin.log_sejak(None).into_iter().map(|b| b.teks).collect();
    assert!(
        log.iter().any(|t| t.contains("seed resmi bawaan dipakai")),
        "{log:?}"
    );
    assert!(
        log.iter().any(|t| t.starts_with("blok diterima: tinggi ")),
        "{log:?}"
    );
    println!(
        "UJI-SUNGGUHAN: sinkron {:?}, blok {} {}",
        node.tinggi_lokal, tambang.blok_terakhir[0].tinggi, tambang.blok_terakhir[0].id_blok
    );
}
