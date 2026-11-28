//! Uji sungguhan modul `vps` dari PC ke VPS (Q199a), dijalankan `q199a_uji.sh` pencipta. Alurnya sama dengan aplikasi Android
//! nanti: masuk dengan password dan sidik kunci host dipercaya, sidik lain ditolak, periksa VPS, pasang paket resmi, kunci
//! aplikasi, skrip bantu tanpa password, terowongan ke pengendali (token salah 401, sinkron, menambang, saldo), perbarui dan
//! penambang dilanjutkan, ganti token, batas CPU, log, amankan-ssh, cabut kunci, copot.
//!
//! Masukan: `ZVRG_UJI_HOST`, `ZVRG_UJI_PORT`, `ZVRG_UJI_PENGGUNA`, `ZVRG_UJI_SIDIK_VPS` (sidik kunci host VPS dari
//! `ssh-keygen -l`, dipisah spasi), `ZVRG_UJI_ALAMAT` (alamat regtest tanpa kunci), dan password pengguna uji di baris pertama
//! stdin. Keluar 0 dengan baris terakhir `UJI-Q199A: SELESAI` bila semua lolos.

#![forbid(unsafe_code)]

use std::process::ExitCode;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use zeroize::Zeroizing;
use zevrigo_node_app_lib::vps::{
    Galat, Kendali, Kunci, Masuk, Sasaran, Sesi, buat_token, pasang_baru, pasang_kunci_publik,
    periksa_vps, sha256_hex,
};

type Hasil<T> = Result<T, String>;

fn env(nama: &str) -> Hasil<String> {
    std::env::var(nama).map_err(|_| format!("{nama} tidak diisi"))
}

fn g(e: Galat) -> String {
    e.to_string()
}

/// Memanggil `metode` setiap 10 detik sampai `cek` terpenuhi atau `menit` habis; keadaan dicetak setiap 60 detik.
async fn tunggu_rpc(
    sesi: &Sesi,
    token: &str,
    metode: &str,
    menit: u64,
    judul: &str,
    cek: impl Fn(&Value) -> bool,
) -> Hasil<Value> {
    let mulai = Instant::now();
    let mut cetak = Instant::now();
    loop {
        match sesi.rpc(token, metode, json!({})).await {
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
                    println!("  {judul}: {e}");
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

async fn uji() -> Hasil<()> {
    let host = env("ZVRG_UJI_HOST")?;
    let port: u16 = env("ZVRG_UJI_PORT")?
        .parse()
        .map_err(|_| "ZVRG_UJI_PORT tidak sah".to_owned())?;
    let pengguna = env("ZVRG_UJI_PENGGUNA")?;
    let sidik_vps = env("ZVRG_UJI_SIDIK_VPS")?;
    let alamat = env("ZVRG_UJI_ALAMAT")?;
    let mut baris = Zeroizing::new(String::new());
    std::io::stdin()
        .read_line(&mut baris)
        .map_err(|e| format!("stdin: {e}"))?;
    let sandi = Zeroizing::new(baris.trim_end_matches(['\r', '\n']).to_owned());
    if sandi.is_empty() {
        return Err("password uji kosong".to_owned());
    }
    let sasaran = Sasaran::baru(&host, port, &pengguna).map_err(g)?;
    println!(
        "sasaran: {}@{}:{}",
        sasaran.pengguna(),
        sasaran.host(),
        sasaran.port()
    );

    // U1 sambungan pertama: password, sidik kunci host dipercaya (harus salah satu kunci host VPS sungguhan).
    let sesi = Sesi::sambung(&sasaran, &Masuk::Sandi(sandi.clone()), None)
        .await
        .map_err(g)?;
    let sidik = sesi.sidik().to_owned();
    if !sidik_vps.split_whitespace().any(|s| s == sidik) {
        return Err(format!(
            "U1: sidik {sidik} bukan kunci host VPS ({sidik_vps})"
        ));
    }
    println!(
        "ok: U1 masuk dengan password, sidik dipercaya pertama kali = kunci host VPS ({sidik})"
    );

    // U2 sidik lain ditolak sebelum password dikirim.
    match Sesi::sambung(
        &sasaran,
        &Masuk::Sandi(sandi.clone()),
        Some("SHA256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
    )
    .await
    {
        Err(Galat::SidikBerubah { .. }) => println!("ok: U2 sidik berbeda ditolak"),
        Err(e) => return Err(format!("U2: galat lain: {e}")),
        Ok(_) => return Err("U2: sidik salah diterima".to_owned()),
    }

    // U3 pemeriksaan VPS.
    let pemeriksaan = periksa_vps(&sesi).await.map_err(g)?;
    println!(
        "periksa: {}",
        serde_json::to_string(&pemeriksaan).unwrap_or_default()
    );
    if !pemeriksaan.masalah().is_empty() || pemeriksaan.terpasang || pemeriksaan.root {
        return Err(format!(
            "U3: VPS tidak siap: {:?}, terpasang {}, root {}",
            pemeriksaan.masalah(),
            pemeriksaan.terpasang,
            pemeriksaan.root
        ));
    }
    println!("ok: U3 periksa VPS ({})", pemeriksaan.nama_os);

    // U4 pemasangan pertama (sudo dengan password, unduh rilis di VPS, SHA-256 tertanam).
    let mut token = buat_token().map_err(g)?;
    let mulai = Instant::now();
    let terpasang = pasang_baru(&sesi, Some(sandi.as_str()), &sha256_hex(&token))
        .await
        .map_err(g)?;
    println!(
        "ok: U4 pasang ({} detik): {terpasang}",
        mulai.elapsed().as_secs()
    );

    // U5 kunci aplikasi, lalu masuk dengan kunci dan sidik tersimpan.
    let kunci = Kunci::baru().map_err(g)?;
    pasang_kunci_publik(&sesi, &kunci).await.map_err(g)?;
    pasang_kunci_publik(&sesi, &kunci).await.map_err(g)?;
    sesi.putus().await;
    let sesi = Sesi::sambung(&sasaran, &kunci.masuk(), Some(&sidik))
        .await
        .map_err(g)?;
    println!(
        "ok: U5 kunci aplikasi dipasang (dua kali, tanpa baris ganda), masuk dengan kunci + sidik tersimpan"
    );

    // U6 skrip bantu lewat sudo -n.
    let status = sesi.status_vps().await.map_err(g)?;
    if status.layanan != "active"
        || !status.versi.contains("e48926b")
        || status.kuota_cpu != "500ms"
    {
        return Err(format!("U6: status tidak sesuai: {status:?}"));
    }
    println!(
        "ok: U6 sudo -n skrip bantu: {} | layanan {} | CPU {}",
        status.versi, status.layanan, status.kuota_cpu
    );

    // U7 terowongan SSH ke pengendali.
    match sesi.rpc(&"x".repeat(64), "info", json!({})).await {
        Err(Galat::TokenDitolak) => {}
        lain => return Err(format!("U7: token salah: {lain:?}")),
    }
    if sesi.rpc(&token, "jalankan", json!({})).await.is_ok() {
        return Err("U7: metode di luar daftar diterima".to_owned());
    }
    let info = sesi.rpc(&token, "info", json!({})).await.map_err(g)?;
    println!(
        "ok: U7 terowongan SSH ke pengendali: token salah 401, metode asing ditolak, info {info}"
    );

    // U8 node VPS sinkron.
    let node = tunggu_rpc(
        &sesi,
        &token,
        "keadaan_node",
        60,
        "U8 node VPS sinkron",
        |v| teks(v, "status") == "sinkron",
    )
    .await?;
    println!(
        "node: tinggi {} | jaringan {} | rekan {}",
        angka(&node, "tinggiLokal"),
        angka(&node, "tinggiJaringan"),
        angka(&node, "rekan")
    );

    // U9 menambang ke alamat uji, saldo dari node VPS.
    let periksa = sesi
        .rpc(&token, "periksa_alamat", json!({"alamat": alamat}))
        .await
        .map_err(g)?;
    if periksa.get("sah").and_then(Value::as_bool) != Some(true) {
        return Err(format!("U9: alamat uji ditolak: {periksa}"));
    }
    sesi.rpc(
        &token,
        "mulai_tambang",
        json!({"alamat": alamat, "jeda": 60}),
    )
    .await
    .map_err(g)?;
    let tambang = tunggu_rpc(
        &sesi,
        &token,
        "keadaan_tambang",
        30,
        "U9 blok pertama penambang VPS",
        |v| angka(v, "blokSesiIni") >= 1,
    )
    .await?;
    let tinggi = tambang
        .get("blokTerakhir")
        .and_then(Value::as_array)
        .and_then(|b| b.first())
        .map_or(0, |b| angka(b, "tinggi"));
    let saldo = sesi
        .rpc(&token, "saldo", json!({"alamat": alamat}))
        .await
        .map_err(g)?;
    if angka(&saldo, "jumlahOutput") < 1 || teks(&saldo, "belumMatang") == "0" {
        return Err(format!("U9: saldo tidak sesuai: {saldo}"));
    }
    println!("ok: U9 blok {tinggi} ke alamat uji, saldo {saldo}");
    println!("TINGGI-BLOK-UJI: {tinggi}");

    // U10 perbarui dari rilis resmi; penambang dilanjutkan sendiri sesudah layanan mulai ulang.
    let perbarui = sesi.kendali(&Kendali::Perbarui).await.map_err(g)?.teks();
    if !perbarui.contains("terpasang: rilis-") || !perbarui.contains("sebelumnya rilis-") {
        return Err(format!("U10: perbarui: {perbarui}"));
    }
    println!("ok: U10 perbarui: {perbarui}");
    tunggu_rpc(
        &sesi,
        &token,
        "info",
        5,
        "U10 pengendali menjawab lagi",
        |_| true,
    )
    .await?;
    tunggu_rpc(
        &sesi,
        &token,
        "keadaan_tambang",
        10,
        "U10 penambang dilanjutkan sendiri",
        |v| teks(v, "alamat") == alamat && teks(v, "status") != "berhenti",
    )
    .await?;
    tunggu_rpc(
        &sesi,
        &token,
        "keadaan_tambang",
        30,
        "U10 blok sesudah dilanjutkan",
        |v| angka(v, "blokSesiIni") >= 1,
    )
    .await?;
    sesi.rpc(&token, "berhenti_tambang", json!({}))
        .await
        .map_err(g)?;
    let tambang = sesi
        .rpc(&token, "keadaan_tambang", json!({}))
        .await
        .map_err(g)?;
    if teks(&tambang, "status") != "berhenti" {
        return Err(format!("U10: penambang tidak berhenti: {tambang}"));
    }
    println!("ok: U10 penambang dihentikan");

    // U11 ganti token.
    let lama = token;
    token = buat_token().map_err(g)?;
    sesi.kendali(&Kendali::Token(sha256_hex(&token)))
        .await
        .map_err(g)?;
    if !matches!(
        sesi.rpc(&lama, "info", json!({})).await,
        Err(Galat::TokenDitolak)
    ) {
        return Err("U11: token lama masih diterima".to_owned());
    }
    sesi.rpc(&token, "info", json!({})).await.map_err(g)?;
    drop(lama);
    println!("ok: U11 token diganti: token lama 401, token baru diterima");

    // U12 batas CPU.
    sesi.kendali(&Kendali::BatasCpu(30)).await.map_err(g)?;
    let s30 = sesi.status_vps().await.map_err(g)?.kuota_cpu;
    if !matches!(
        sesi.kendali(&Kendali::BatasCpu(5)).await,
        Err(Galat::Masukan(_))
    ) {
        return Err("U12: batas CPU 5 tidak ditolak aplikasi".to_owned());
    }
    sesi.kendali(&Kendali::BatasCpu(50)).await.map_err(g)?;
    let s50 = sesi.status_vps().await.map_err(g)?.kuota_cpu;
    if s30 != "300ms" || s50 != "500ms" {
        return Err(format!("U12: batas CPU {s30} / {s50}"));
    }
    println!("ok: U12 batas CPU 30% ({s30}) lalu 50% ({s50})");

    // U13 log.
    let log = sesi
        .rpc(&token, "log_terbaru", json!({}))
        .await
        .map_err(g)?;
    let jumlah = log
        .as_array()
        .or_else(|| {
            log.as_object()
                .and_then(|o| o.values().find_map(Value::as_array))
        })
        .map_or(0, Vec::len);
    if jumlah == 0 {
        return Err(format!("U13: log kosong: {log}"));
    }
    println!("ok: U13 log {jumlah} baris");

    // U14 amankan-ssh (pengguna uji sudah punya kunci aplikasi).
    sesi.kendali(&Kendali::AmankanSsh).await.map_err(g)?;
    let status = sesi.status_vps().await.map_err(g)?;
    if status.login_password != "no" {
        return Err(format!(
            "U14: login password global {}",
            status.login_password
        ));
    }
    println!(
        "ok: U14 amankan-ssh: login password global {}",
        status.login_password
    );

    // U15 cabut kunci aplikasi: masuk dengan kunci ditolak; dipasang lagi lewat password pengguna uji (blok Match uji).
    sesi.kendali(&Kendali::CabutKunci(kunci.blob_publik().map_err(g)?))
        .await
        .map_err(g)?;
    sesi.putus().await;
    match Sesi::sambung(&sasaran, &kunci.masuk(), Some(&sidik)).await {
        Err(Galat::MasukDitolak) => {}
        lain => return Err(format!("U15: kunci dicabut tetapi: {lain:?}")),
    }
    let sesi = Sesi::sambung(&sasaran, &Masuk::Sandi(sandi.clone()), Some(&sidik))
        .await
        .map_err(g)?;
    pasang_kunci_publik(&sesi, &kunci).await.map_err(g)?;
    sesi.putus().await;
    let sesi = Sesi::sambung(&sasaran, &kunci.masuk(), Some(&sidik))
        .await
        .map_err(g)?;
    println!("ok: U15 cabut kunci: masuk dengan kunci ditolak, dipasang lagi, masuk lagi");

    // U16 copot dengan data.
    let copot = sesi
        .kendali(&Kendali::Copot { hapus_data: true })
        .await
        .map_err(g)?
        .teks();
    let akhir = periksa_vps(&sesi).await.map_err(g)?;
    if akhir.terpasang {
        return Err(format!("U16: masih terpasang sesudah copot: {copot}"));
    }
    sesi.putus().await;
    println!("ok: U16 copot --hapus-data: {copot}");
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
            println!("UJI-Q199A: SELESAI");
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("GAGAL: {e}");
            ExitCode::FAILURE
        }
    }
}
