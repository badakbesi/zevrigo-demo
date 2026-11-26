//! Uji sungguhan layanan VPS aplikasi (Q199b) dari PC ke VPS, dijalankan `q199b_uji.sh` pencipta. Alurnya sama dengan
//! layar aplikasi Android: buat kata sandi aplikasi, sambung dengan password, pasang, amankan, kunci dan buka, pengendali
//! (sinkron, tambang, saldo), perbarui, ganti token, batas CPU, ekspor dan impor (pindah HP), ambil alih node yang sudah
//! terpasang (kata sandi aplikasi lupa), cabut akses, copot.
//!
//! Masukan: `ZVRG_UJI_HOST`, `ZVRG_UJI_PORT`, `ZVRG_UJI_PENGGUNA`, `ZVRG_UJI_SIDIK_VPS` (sidik kunci host, dipisah spasi),
//! `ZVRG_UJI_ALAMAT` (alamat regtest tanpa kunci), `ZVRG_UJI_BRANKAS` (berkas brankas sementara), dan password pengguna uji
//! di baris pertama stdin. Keluar 0 dengan baris terakhir `UJI-Q199B: SELESAI` bila semua lolos.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use zeroize::Zeroizing;
use zevrigo_node_app_lib::vps::{
    Galat, Kendali, Layanan, Masuk, PerintahUi, Sasaran, Sesi, Tahap, periksa_vps,
};

type Hasil<T> = Result<T, String>;

fn env(nama: &str) -> Hasil<String> {
    std::env::var(nama).map_err(|_| format!("{nama} tidak diisi"))
}

fn g(e: Galat) -> String {
    format!("{} ({})", e, e.kode())
}

fn teks(nilai: &Value, kunci: &str) -> String {
    nilai
        .get(kunci)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn angka(nilai: &Value, kunci: &str) -> u64 {
    nilai.get(kunci).and_then(Value::as_u64).unwrap_or(0)
}

/// Memanggil `metode` setiap 10 detik sampai `cek` terpenuhi atau `menit` habis; keadaan dicetak setiap 60 detik.
async fn tunggu(
    l: &Layanan,
    metode: &str,
    menit: u64,
    judul: &str,
    cek: impl Fn(&Value) -> bool,
) -> Hasil<Value> {
    let mulai = Instant::now();
    let mut cetak = Instant::now();
    loop {
        match l.rpc(metode, json!({})).await {
            Ok(nilai) if cek(&nilai) => {
                println!("ok: {judul} ({} detik)", mulai.elapsed().as_secs());
                return Ok(nilai);
            }
            Ok(nilai) => {
                if cetak.elapsed() >= Duration::from_secs(60) {
                    println!("  {judul}: {nilai}");
                    cetak = Instant::now();
                }
            }
            Err(e) => {
                if cetak.elapsed() >= Duration::from_secs(60) {
                    println!("  {judul}: {}", g(e));
                    cetak = Instant::now();
                }
            }
        }
        if mulai.elapsed() >= Duration::from_secs(menit.saturating_mul(60)) {
            return Err(format!("waktu habis: {judul}"));
        }
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

fn wajib_tahap(langkah: &str, tahap: Tahap, harap: Tahap) -> Hasil<()> {
    if tahap == harap {
        Ok(())
    } else {
        Err(format!("{langkah}: tahap {tahap:?}, harus {harap:?}"))
    }
}

/// Sambung dengan password + pasang (pemasangan baru atau ambil alih) + amankan; mengembalikan apakah VPS sudah terpasang.
async fn pasang_penuh(
    l: &Layanan,
    langkah: &str,
    host: &str,
    port: u16,
    pengguna: &str,
    sandi_vps: &str,
    sandi_aplikasi: &str,
) -> Hasil<bool> {
    wajib_tahap(
        langkah,
        l.buat_sandi(sandi_aplikasi).map_err(g)?.tahap,
        Tahap::Sambung,
    )?;
    let info = l
        .sambung_awal(host, port, pengguna, sandi_vps)
        .await
        .map_err(g)?;
    wajib_tahap(langkah, info.tahap, Tahap::Pasang)?;
    let pemeriksaan = info.pemeriksaan.ok_or("pemeriksaan kosong")?;
    if !info.masalah.is_empty() {
        return Err(format!("{langkah}: VPS tidak siap: {:?}", info.masalah));
    }
    let mulai = Instant::now();
    wajib_tahap(langkah, l.pasang().await.map_err(g)?.tahap, Tahap::Amankan)?;
    println!(
        "ok: {langkah} pasang ({} detik, {})",
        mulai.elapsed().as_secs(),
        if pemeriksaan.terpasang {
            "node yang sudah ada diambil alih dengan token baru"
        } else {
            "pemasangan baru"
        }
    );
    let info = l.amankan().await.map_err(g)?;
    wajib_tahap(langkah, info.tahap, Tahap::Siap)?;
    if !info.amankan_selesai {
        return Err(format!("{langkah}: amankan tidak tercatat"));
    }
    Ok(pemeriksaan.terpasang)
}

#[allow(clippy::too_many_lines)]
async fn uji() -> Hasil<()> {
    let host = env("ZVRG_UJI_HOST")?;
    let port: u16 = env("ZVRG_UJI_PORT")?
        .parse()
        .map_err(|_| "ZVRG_UJI_PORT tidak sah".to_owned())?;
    let pengguna = env("ZVRG_UJI_PENGGUNA")?;
    let sidik_vps = env("ZVRG_UJI_SIDIK_VPS")?;
    let alamat = env("ZVRG_UJI_ALAMAT")?;
    let berkas = PathBuf::from(env("ZVRG_UJI_BRANKAS")?);
    let mut baris = Zeroizing::new(String::new());
    std::io::stdin()
        .read_line(&mut baris)
        .map_err(|e| format!("stdin: {e}"))?;
    let sandi_vps = Zeroizing::new(baris.trim_end_matches(['\r', '\n']).to_owned());
    if sandi_vps.is_empty() {
        return Err("password uji kosong".to_owned());
    }
    let sandi_app = "sandi-aplikasi-uji-1";
    let _ = std::fs::remove_file(&berkas);
    // Argon2id bawaan wallet (64 MiB, 3 putaran), sama dengan aplikasi.
    let l = Layanan::baru(berkas.clone());
    println!("brankas uji: {}", berkas.display());

    // L1-L2 kata sandi aplikasi, sambungan pertama dengan password, sidik kunci host = kunci VPS sungguhan.
    wajib_tahap("L1", l.info().map_err(g)?.tahap, Tahap::Baru)?;
    if !matches!(l.buat_sandi("pendek"), Err(Galat::Masukan(_))) {
        return Err("L1: sandi 6 karakter tidak ditolak".to_owned());
    }
    wajib_tahap(
        "L1",
        l.buat_sandi(sandi_app).map_err(g)?.tahap,
        Tahap::Sambung,
    )?;
    let info = l
        .sambung_awal(&host, port, &pengguna, &sandi_vps)
        .await
        .map_err(g)?;
    wajib_tahap("L2", info.tahap, Tahap::Pasang)?;
    let sidik = info.sidik.clone().unwrap_or_default();
    if !sidik_vps.split_whitespace().any(|s| s == sidik) {
        return Err(format!("L2: sidik {sidik} bukan kunci host VPS"));
    }
    let terpasang = info.pemeriksaan.as_ref().is_some_and(|p| p.terpasang);
    if !info.masalah.is_empty() || terpasang {
        return Err(format!(
            "L2: VPS tidak siap: {:?}, terpasang {terpasang}",
            info.masalah
        ));
    }
    println!("ok: L1-L2 kata sandi aplikasi, sambung dengan password, sidik {sidik}");

    // L3 pasang: brankas tertulis, tanpa rahasia terbaca.
    let mulai = Instant::now();
    wajib_tahap("L3", l.pasang().await.map_err(g)?.tahap, Tahap::Amankan)?;
    let isi = std::fs::read_to_string(&berkas).map_err(|e| format!("L3: brankas: {e}"))?;
    if isi.contains("OPENSSH") || isi.contains(host.as_str()) || isi.contains(pengguna.as_str()) {
        return Err("L3: brankas memuat rahasia terbaca".to_owned());
    }
    println!(
        "ok: L3 pasang ({} detik), brankas {} B tanpa rahasia terbaca",
        mulai.elapsed().as_secs(),
        isi.len()
    );

    // L4 skrip bantu + pengendali lewat kunci aplikasi.
    let status = l.status().await.map_err(g)?;
    if status.layanan != "active" || !status.versi.contains("faa9c3a") {
        return Err(format!("L4: status: {status:?}"));
    }
    let info_rpc = l.rpc("info", json!({})).await.map_err(g)?;
    println!(
        "ok: L4 status {} | {} | pengendali {info_rpc}",
        status.layanan, status.versi
    );

    // L5 amankan.
    let info = l.amankan().await.map_err(g)?;
    wajib_tahap("L5", info.tahap, Tahap::Siap)?;
    let status = l.status().await.map_err(g)?;
    println!(
        "ok: L5 amankan: login password global {}",
        status.login_password
    );

    // L6 kunci dan buka.
    wajib_tahap("L6", l.kunci().await.map_err(g)?.tahap, Tahap::Terkunci)?;
    if l.rpc("info", json!({})).await != Err(Galat::Terkunci) {
        return Err("L6: pengendali bisa dipanggil saat terkunci".to_owned());
    }
    if l.buka("sandi-salah-sekali") != Err(Galat::SandiSalah) {
        return Err("L6: sandi salah tidak ditolak".to_owned());
    }
    let info = l.buka(sandi_app).map_err(g)?;
    wajib_tahap("L6", info.tahap, Tahap::Siap)?;
    println!(
        "ok: L6 kunci: pengendali ditolak, sandi salah ditolak, buka lagi (amankan tersimpan)"
    );

    // L7-L8 sinkron, pengaturan, tambang, saldo.
    let node = tunggu(&l, "keadaan_node", 60, "L7 node VPS sinkron", |v| {
        teks(v, "status") == "sinkron"
    })
    .await?;
    println!(
        "node: tinggi {} | rekan {}",
        angka(&node, "tinggiLokal"),
        angka(&node, "rekan")
    );
    l.rpc(
        "simpan_pengaturan",
        json!({"pengaturan": {"alamatHadiah": alamat, "jedaDetik": 60, "tema": "sistem"}}),
    )
    .await
    .map_err(g)?;
    let p = l.rpc("muat_pengaturan", json!({})).await.map_err(g)?;
    if teks(&p, "alamatHadiah") != alamat || angka(&p, "jedaDetik") != 60 {
        return Err(format!("L8: pengaturan: {p}"));
    }
    l.rpc("mulai_tambang", json!({"alamat": alamat, "jeda": 60}))
        .await
        .map_err(g)?;
    let tambang = tunggu(&l, "keadaan_tambang", 30, "L8 blok pertama", |v| {
        angka(v, "blokSesiIni") >= 1
    })
    .await?;
    let tinggi = tambang
        .get("blokTerakhir")
        .and_then(Value::as_array)
        .and_then(|b| b.first())
        .map_or(0, |b| angka(b, "tinggi"));
    let saldo = l.rpc("saldo", json!({"alamat": alamat})).await.map_err(g)?;
    if angka(&saldo, "jumlahOutput") < 1 || teks(&saldo, "belumMatang") == "0" {
        return Err(format!("L8: saldo: {saldo}"));
    }
    println!("ok: L8 pengaturan tersimpan di VPS, blok {tinggi}, saldo {saldo}");
    println!("TINGGI-BLOK-UJI: {tinggi}");

    // L9 perbarui (layanan mulai ulang; sambungan pengendali dibuat ulang sendiri), penambang dilanjutkan.
    let kendali = PerintahUi::Perbarui.ke_kendali().map_err(g)?;
    let keluaran = l.kendali(&kendali).await.map_err(g)?.teks();
    if !keluaran.contains("terpasang: rilis-") {
        return Err(format!("L9: perbarui: {keluaran}"));
    }
    tunggu(&l, "keadaan_tambang", 10, "L9 penambang dilanjutkan", |v| {
        teks(v, "alamat") == alamat && teks(v, "status") != "berhenti"
    })
    .await?;
    tunggu(
        &l,
        "keadaan_tambang",
        30,
        "L9 blok sesudah dilanjutkan",
        |v| angka(v, "blokSesiIni") >= 1,
    )
    .await?;
    l.rpc("berhenti_tambang", json!({})).await.map_err(g)?;
    println!("ok: L9 perbarui: {keluaran}");

    // L10 ganti token: tersimpan di brankas.
    l.ganti_token().await.map_err(g)?;
    l.rpc("info", json!({})).await.map_err(g)?;
    l.kunci().await.map_err(g)?;
    l.buka(sandi_app).map_err(g)?;
    l.rpc("info", json!({})).await.map_err(g)?;
    println!("ok: L10 token diganti dan tersimpan (kunci + buka, pengendali menjawab)");

    // L11 batas CPU dan node publik mati.
    for (persen, harap) in [(30_u8, "300ms"), (50, "500ms")] {
        l.kendali(&Kendali::BatasCpu(persen)).await.map_err(g)?;
        let kuota = l.status().await.map_err(g)?.kuota_cpu;
        if kuota != harap {
            return Err(format!("L11: batas CPU {persen}: {kuota}"));
        }
    }
    l.kendali(&Kendali::NodePublikMati).await.map_err(g)?;
    println!("ok: L11 batas CPU 30% lalu 50%, node publik mati");

    // L12 ekspor, hapus dari HP, impor (pindah HP).
    let ekspor = l.ekspor().map_err(g)?;
    wajib_tahap("L12", l.lupakan().await.map_err(g)?.tahap, Tahap::Baru)?;
    if berkas.exists() {
        return Err("L12: brankas masih ada sesudah dihapus".to_owned());
    }
    if l.impor(&ekspor, "sandi-salah-sekali") != Err(Galat::SandiSalah) {
        return Err("L12: impor dengan sandi salah tidak ditolak".to_owned());
    }
    wajib_tahap(
        "L12",
        l.impor(&ekspor, sandi_app).map_err(g)?.tahap,
        Tahap::Siap,
    )?;
    l.rpc("info", json!({})).await.map_err(g)?;
    println!(
        "ok: L12 ekspor {} B, hapus dari HP, impor: pengendali menjawab",
        ekspor.len()
    );

    // L13 ambil alih (kata sandi aplikasi lupa): hapus data HP, sambung ulang dengan password, token baru.
    l.lupakan().await.map_err(g)?;
    let ambil = pasang_penuh(
        &l,
        "L13",
        &host,
        port,
        &pengguna,
        &sandi_vps,
        "sandi-aplikasi-uji-2",
    )
    .await?;
    if !ambil {
        return Err("L13: VPS tidak terdeteksi sudah terpasang".to_owned());
    }
    l.rpc("info", json!({})).await.map_err(g)?;
    println!("ok: L13 ambil alih: node yang sama, token baru, pengendali menjawab");

    // L14 cabut akses: kunci HP ini dihapus dari VPS; data lama (ekspor) tidak bisa masuk lagi.
    let ekspor = l.ekspor().map_err(g)?;
    wajib_tahap("L14", l.cabut_akses().await.map_err(g)?.tahap, Tahap::Baru)?;
    l.impor(&ekspor, "sandi-aplikasi-uji-2").map_err(g)?;
    match l.rpc("info", json!({})).await {
        Err(Galat::MasukDitolak) => {}
        lain => return Err(format!("L14: kunci dicabut tetapi: {lain:?}")),
    }
    l.lupakan().await.map_err(g)?;
    println!("ok: L14 cabut akses: kunci lama ditolak VPS");

    // L15 copot lewat aplikasi (sesudah ambil alih lagi), lalu VPS diperiksa: tidak terpasang.
    pasang_penuh(
        &l,
        "L15",
        &host,
        port,
        &pengguna,
        &sandi_vps,
        "sandi-aplikasi-uji-3",
    )
    .await?;
    wajib_tahap("L15", l.copot(true).await.map_err(g)?.tahap, Tahap::Baru)?;
    let sasaran = Sasaran::baru(&host, port, &pengguna).map_err(g)?;
    let sesi = Sesi::sambung(&sasaran, &Masuk::Sandi(sandi_vps.clone()), Some(&sidik))
        .await
        .map_err(g)?;
    let akhir = periksa_vps(&sesi).await.map_err(g)?;
    sesi.putus().await;
    if akhir.terpasang || berkas.exists() {
        return Err(format!(
            "L15: terpasang {}, brankas {}",
            akhir.terpasang,
            berkas.exists()
        ));
    }
    println!("ok: L15 copot --hapus-data lewat aplikasi, data VPS di HP dihapus");
    Ok(())
}

fn main() -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => {
            println!("GAGAL: runtime: {e}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(uji()) {
        Ok(()) => {
            println!("UJI-Q199B: SELESAI");
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("GAGAL: {e}");
            ExitCode::FAILURE
        }
    }
}
