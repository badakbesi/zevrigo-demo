//! Mesin aplikasi Zevrigo Node (RENCANA_T16 bagian 8, DEC-44), tanpa tipe Tauri agar bisa diuji sendiri.
//!
//! - Node: proses anak `zevrigo-node --regtest --datadir <folder> --seed-resmi`; keluarannya masuk log, jumlah rekan dihitung
//!   dari baris `rekan terhubung` / `rekan terputus`. Bila RPC 127.0.0.1:28801 sudah menjawab sebelum dinyalakan, node lain itu
//!   dipakai apa adanya (seperti skrip awal paket peserta) dan tidak dimatikan aplikasi.
//! - Pemantau: tinggi lokal dari RPC node setiap 2 detik, tinggi jaringan dari gerbang RPC publik setiap 15 detik (MOD-12).
//! - Penambang: putaran `zevrigo-miner --payout <payload> --payout-kind <k> --blocks 1`, baris `blok diterima: tinggi N, ...,
//!   id X` dicatat, lalu jeda 60, 120, atau 300 detik.
//! - Alamat hadiah diurai MOD-09 (`decode_address`) menjadi payload. Tidak ada kunci rahasia di aplikasi.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use zevrigo_cli::Cli;
use zevrigo_primitives::Network;
use zevrigo_wallet_core::{AddressError, decode_address};

/// RPC node lokal (bawaan regtest node dan miner).
pub(crate) const RPC_LOKAL: &str = "http://127.0.0.1:28801";
/// Gerbang RPC publik jaringan demo (tinggi jaringan).
pub(crate) const RPC_PUBLIK: &str = "https://demo-gerbang-rpc.zevrigo.online";
/// Baris log paling banyak yang disimpan.
pub(crate) const MAKS_LOG: usize = 500;
/// Blok terakhir paling banyak yang ditampilkan.
pub(crate) const MAKS_BLOK: usize = 5;
/// Nama berkas pengaturan di folder data.
pub(crate) const BERKAS_PENGATURAN: &str = "pengaturan-zevrigo-node.json";
/// Jeda antar blok yang diizinkan (pencipta 23 November 2026: bawaan 120 detik).
pub(crate) const JEDA_SAH: [u64; 3] = [60, 120, 300];
/// Batas waktu menunggu node berhenti sendiri sebelum dihentikan paksa.
const TUNGGU_BERHENTI: Duration = Duration::from_secs(30);
/// Windows: proses anak tanpa jendela konsol (`CREATE_NO_WINDOW`).
#[cfg(windows)]
const TANPA_JENDELA: u32 = 0x0800_0000;

/// Waktu sekarang dalam milidetik Unix.
fn sekarang_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Mengunci mutex; mutex yang beracun (utas lain panik) tetap dipakai.
fn kunci<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Keadaan node untuk antarmuka (`KeadaanNode` di src/lib/backend.ts).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KeadaanNode {
    pub(crate) status: &'static str,
    pub(crate) tinggi_lokal: Option<u64>,
    pub(crate) tinggi_jaringan: Option<u64>,
    pub(crate) rekan: u32,
    pub(crate) folder_data: String,
    pub(crate) pesan_galat: Option<String>,
    pub(crate) detail_galat: Option<String>,
}

/// Blok yang ditemukan penambang.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlokDitemukan {
    pub(crate) tinggi: u64,
    pub(crate) id_blok: String,
    pub(crate) waktu: u64,
}

/// Keadaan penambang untuk antarmuka.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KeadaanTambang {
    pub(crate) status: &'static str,
    pub(crate) alamat: Option<String>,
    pub(crate) jeda_detik: u64,
    pub(crate) sisa_jeda_detik: Option<u64>,
    pub(crate) blok_sesi_ini: u64,
    pub(crate) blok_terakhir: Vec<BlokDitemukan>,
    pub(crate) pesan_galat: Option<String>,
    pub(crate) detail_galat: Option<String>,
}

/// Satu baris log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct BarisLog {
    pub(crate) waktu: u64,
    pub(crate) sumber: &'static str,
    pub(crate) tingkat: &'static str,
    pub(crate) teks: String,
}

/// Pengaturan yang disimpan di folder data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct Pengaturan {
    pub(crate) alamat_hadiah: String,
    pub(crate) jeda_detik: u64,
    pub(crate) tema: String,
}

impl Default for Pengaturan {
    fn default() -> Self {
        Self {
            alamat_hadiah: String::new(),
            jeda_detik: 120,
            tema: "sistem".to_owned(),
        }
    }
}

impl Pengaturan {
    /// Pengaturan dengan nilai di luar pilihan dikembalikan ke bawaan.
    pub(crate) fn rapikan(mut self) -> Self {
        if !JEDA_SAH.contains(&self.jeda_detik) {
            self.jeda_detik = 120;
        }
        if !["sistem", "terang", "gelap"].contains(&self.tema.as_str()) {
            self.tema = "sistem".to_owned();
        }
        self.alamat_hadiah = self.alamat_hadiah.trim().chars().take(200).collect();
        self
    }

    /// Membaca isi berkas pengaturan; JSON rusak menghasilkan bawaan.
    pub(crate) fn dari_json(teks: &str) -> Self {
        serde_json::from_str::<Self>(teks)
            .unwrap_or_default()
            .rapikan()
    }
}

/// Hasil pemeriksaan alamat hadiah.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct HasilPeriksaAlamat {
    pub(crate) sah: bool,
    pub(crate) alasan: Option<&'static str>,
}

/// Mengurai alamat hadiah regtest menjadi hasil pemeriksaan dan, bila sah, `(jenis output, payload hex)` untuk miner.
pub(crate) fn periksa_alamat(alamat: &str) -> (HasilPeriksaAlamat, Option<(u8, String)>) {
    let alamat = alamat.trim();
    if alamat.is_empty() {
        let hasil = HasilPeriksaAlamat {
            sah: false,
            alasan: Some("kosong"),
        };
        return (hasil, None);
    }
    match decode_address(Network::Regtest, alamat) {
        Ok((jenis, payload)) => {
            let mut hex = String::with_capacity(payload.len().saturating_mul(2));
            for b in payload {
                let _ = write!(hex, "{b:02x}");
            }
            let hasil = HasilPeriksaAlamat {
                sah: true,
                alasan: None,
            };
            (hasil, Some((jenis.to_u8(), hex)))
        }
        Err(AddressError::WrongNetwork) => {
            let hasil = HasilPeriksaAlamat {
                sah: false,
                alasan: Some("jaringan-lain"),
            };
            (hasil, None)
        }
        Err(_) => {
            let hasil = HasilPeriksaAlamat {
                sah: false,
                alasan: Some("format"),
            };
            (hasil, None)
        }
    }
}

/// Perubahan jumlah rekan dari satu baris log node.
pub(crate) fn rekan_dari_baris(baris: &str) -> Option<i8> {
    if baris.contains("rekan terhubung") {
        Some(1)
    } else if baris.contains("rekan terputus") {
        Some(-1)
    } else {
        None
    }
}

/// `(tinggi, id blok)` dari baris `blok diterima: tinggi N, nonce K, id X` keluaran `zevrigo-miner`.
pub(crate) fn blok_dari_baris(baris: &str) -> Option<(u64, String)> {
    let sisa = baris.trim().strip_prefix("blok diterima: tinggi ")?;
    let (tinggi, sisa) = sisa.split_once(',')?;
    let tinggi = tinggi.trim().parse::<u64>().ok()?;
    let (_, id) = sisa.split_once(", id ")?;
    let id = id.trim();
    (id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit()))
        .then(|| (tinggi, id.to_ascii_lowercase()))
}

/// Tingkat baris log (`info`, `peringatan`, `galat`): dari tingkat tracing node bila ada, selain itu dari kata `galat`/`gagal`.
pub(crate) fn tingkat_baris(baris: &str) -> &'static str {
    if baris.contains(" ERROR ") {
        "galat"
    } else if baris.contains(" WARN ") {
        "peringatan"
    } else if baris.contains(" INFO ") || baris.contains(" DEBUG ") || baris.contains(" TRACE ") {
        "info"
    } else if baris.contains("galat") || baris.contains("gagal") {
        "galat"
    } else {
        "info"
    }
}

/// Baris keluaran tanpa kode warna terminal (`ESC [ ... huruf`) dan karakter kendali lain.
pub(crate) fn bersihkan_baris(baris: &str) -> String {
    let mut hasil = String::with_capacity(baris.len());
    let mut karakter = baris.chars().peekable();
    while let Some(c) = karakter.next() {
        if c == '\u{1b}' {
            if karakter.peek() == Some(&'[') {
                karakter.next();
                for d in karakter.by_ref() {
                    if d.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else if c == '\t' {
            hasil.push(' ');
        } else if !c.is_control() {
            hasil.push(c);
        }
    }
    hasil
}

/// `true` bila `url` adalah tautan resmi yang boleh dibuka aplikasi di browser sistem.
pub(crate) fn tautan_resmi(url: &str) -> bool {
    url == "https://zevrigo.online/"
        || url.starts_with("https://zevrigo.online/unduh/")
        || url.starts_with("https://zevrigo.online/artikel/")
        || url == "https://demo-explorer.zevrigo.online"
        || url.starts_with("https://demo-explorer.zevrigo.online/")
        || url == "https://github.com/badakbesi/zevrigo-demo/issues"
}

/// Status node dari pengamatan (lihat [`Mesin::keadaan_node`]).
pub(crate) fn status_node(
    berjalan: bool,
    luar: bool,
    galat: bool,
    tinggi_lokal: Option<u64>,
    tinggi_jaringan: Option<u64>,
    rekan: u32,
) -> &'static str {
    if galat {
        return "galat";
    }
    if !berjalan && !luar {
        return "mati";
    }
    let Some(lokal) = tinggi_lokal else {
        return if luar { "menyambung" } else { "menyala" };
    };
    if rekan == 0 && !luar {
        return "menyambung";
    }
    match tinggi_jaringan {
        Some(jaringan) if lokal >= jaringan => "sinkron",
        Some(_) => "mengejar",
        None => "sinkron",
    }
}

/// Penampung log dengan waktu yang selalu naik (antarmuka meminta baris sesudah waktu terakhir yang ia terima).
#[derive(Debug, Default)]
pub(crate) struct PenampungLog {
    baris: VecDeque<BarisLog>,
    terakhir: u64,
}

impl PenampungLog {
    pub(crate) fn tambah(&mut self, sumber: &'static str, tingkat: &'static str, teks: &str) {
        let waktu = sekarang_ms().max(self.terakhir.saturating_add(1));
        self.terakhir = waktu;
        self.baris.push_back(BarisLog {
            waktu,
            sumber,
            tingkat,
            teks: teks.chars().take(2000).collect(),
        });
        while self.baris.len() > MAKS_LOG {
            self.baris.pop_front();
        }
    }

    pub(crate) fn sejak(&self, waktu: Option<u64>) -> Vec<BarisLog> {
        self.baris
            .iter()
            .filter(|b| waktu.is_none_or(|w| b.waktu > w))
            .cloned()
            .collect()
    }

    fn ekor(&self, jumlah: usize, sumber: &str) -> String {
        let pilih: Vec<&str> = self
            .baris
            .iter()
            .filter(|b| b.sumber == sumber)
            .map(|b| b.teks.as_str())
            .collect();
        let mulai = pilih.len().saturating_sub(jumlah);
        pilih.get(mulai..).unwrap_or_default().join("\n")
    }
}

/// Pengamatan node dari pemantau dan pembaca keluaran.
#[derive(Debug, Default)]
struct Pantau {
    tinggi_lokal: Option<u64>,
    tinggi_jaringan: Option<u64>,
    rekan: u32,
    galat: Option<(String, String)>,
}

/// Keadaan penambang yang dibagi utas penambang dan perintah.
#[derive(Debug)]
struct Tambang {
    status: &'static str,
    alamat: Option<String>,
    jeda: u64,
    sisa: Option<u64>,
    blok_sesi: u64,
    blok: Vec<BlokDitemukan>,
    galat: Option<(String, String)>,
}

impl Default for Tambang {
    fn default() -> Self {
        Self {
            status: "berhenti",
            alamat: None,
            jeda: 120,
            sisa: None,
            blok_sesi: 0,
            blok: Vec::new(),
            galat: None,
        }
    }
}

/// Mesin aplikasi: satu per proses, dibagi semua perintah lewat `Arc`.
#[derive(Debug)]
pub(crate) struct Mesin {
    folder: PathBuf,
    bin_node: PathBuf,
    bin_miner: PathBuf,
    node: Mutex<Option<Child>>,
    node_luar: AtomicBool,
    pantau: Mutex<Pantau>,
    log: Mutex<PenampungLog>,
    tambang: Mutex<Tambang>,
    /// Naik setiap kali penambangan dimulai atau dihentikan; utas penambang lama berhenti bila nilainya berubah.
    putaran: AtomicU64,
    miner: Mutex<Option<Child>>,
    akan_tutup: AtomicBool,
}

/// Folder data bawaan, sama dengan paket peserta: `ZVRG_DATA`, atau `%USERPROFILE%\.zevrigo` (Windows) / `~/.zevrigo`.
pub(crate) fn folder_data_bawaan() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("ZVRG_DATA").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    let rumah = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var_os(rumah)
        .filter(|d| !d.is_empty())
        .map(|d| PathBuf::from(d).join(".zevrigo"))
}

/// Path sidecar `nama` di samping program aplikasi (Tauri `externalBin`), atau dari variabel lingkungan `ganti` (uji).
pub(crate) fn bin_sidecar(nama: &str, ganti: &str) -> PathBuf {
    if let Some(path) = std::env::var_os(ganti).filter(|p| !p.is_empty()) {
        return PathBuf::from(path);
    }
    let berkas = if cfg!(windows) {
        format!("{nama}.exe")
    } else {
        nama.to_owned()
    };
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_default()
        .join(berkas)
}

/// Menjalankan `program` tanpa jendela konsol di Windows.
fn perintah(program: &Path) -> Command {
    let mut cmd = Command::new(program);
    sembunyikan_jendela(&mut cmd);
    cmd
}

#[cfg(windows)]
fn sembunyikan_jendela(cmd: &mut Command) {
    use std::os::windows::process::CommandExt as _;
    cmd.creation_flags(TANPA_JENDELA);
}

#[cfg(not(windows))]
fn sembunyikan_jendela(_cmd: &mut Command) {}

/// Tinggi chain node klien `cli`, atau `None` bila tidak menjawab.
fn tinggi_di(cli: Option<&Cli>) -> Option<u64> {
    cli.and_then(|c| c.info().ok()).map(|info| info.height)
}

impl Mesin {
    pub(crate) fn baru(folder: PathBuf, bin_node: PathBuf, bin_miner: PathBuf) -> Self {
        Self {
            folder,
            bin_node,
            bin_miner,
            node: Mutex::new(None),
            node_luar: AtomicBool::new(false),
            pantau: Mutex::new(Pantau::default()),
            log: Mutex::new(PenampungLog::default()),
            tambang: Mutex::new(Tambang::default()),
            putaran: AtomicU64::new(0),
            miner: Mutex::new(None),
            akan_tutup: AtomicBool::new(false),
        }
    }

    pub(crate) fn folder(&self) -> &Path {
        &self.folder
    }

    pub(crate) fn catat(&self, sumber: &'static str, teks: &str) {
        kunci(&self.log).tambah(sumber, tingkat_baris(teks), teks);
    }

    pub(crate) fn log_sejak(&self, waktu: Option<u64>) -> Vec<BarisLog> {
        kunci(&self.log).sejak(waktu)
    }

    /// Utas pemantau: tinggi lokal setiap 2 detik, tinggi jaringan setiap 15 detik, dan node yang berhenti sendiri.
    pub(crate) fn mulai_pemantau(self: &Arc<Self>) {
        let mesin = Arc::clone(self);
        thread::spawn(move || {
            let lokal = Cli::connect(RPC_LOKAL, Network::Regtest).ok();
            let publik = Cli::connect(RPC_PUBLIK, Network::Regtest).ok();
            let mut publik_terakhir: Option<Instant> = None;
            loop {
                mesin.periksa_node_berhenti();
                let aktif = kunci(&mesin.node).is_some() || mesin.node_luar.load(Ordering::SeqCst);
                let tinggi = if aktif {
                    tinggi_di(lokal.as_ref())
                } else {
                    None
                };
                kunci(&mesin.pantau).tinggi_lokal = tinggi;
                if aktif && publik_terakhir.is_none_or(|t| t.elapsed() >= Duration::from_secs(15)) {
                    publik_terakhir = Some(Instant::now());
                    if let Some(j) = tinggi_di(publik.as_ref()) {
                        kunci(&mesin.pantau).tinggi_jaringan = Some(j);
                    }
                }
                thread::sleep(Duration::from_secs(2));
            }
        });
    }

    /// Node yang dinyalakan aplikasi tetapi berhenti sendiri: status galat dengan ekor log node.
    fn periksa_node_berhenti(&self) {
        let mut node = kunci(&self.node);
        let selesai = node.as_mut().and_then(|c| c.try_wait().ok().flatten());
        if let Some(kode) = selesai {
            *node = None;
            drop(node);
            let detail = kunci(&self.log).ekor(15, "node");
            let mut pantau = kunci(&self.pantau);
            pantau.rekan = 0;
            pantau.tinggi_lokal = None;
            pantau.galat = Some((format!("Node berhenti sendiri ({kode})."), detail));
            drop(pantau);
            self.hentikan_tambang();
            self.catat("aplikasi", "node berhenti sendiri");
        }
    }

    pub(crate) fn keadaan_node(&self) -> KeadaanNode {
        let berjalan = kunci(&self.node).is_some();
        let luar = self.node_luar.load(Ordering::SeqCst);
        let pantau = kunci(&self.pantau);
        let status = status_node(
            berjalan,
            luar,
            pantau.galat.is_some(),
            pantau.tinggi_lokal,
            pantau.tinggi_jaringan,
            pantau.rekan,
        );
        KeadaanNode {
            status,
            tinggi_lokal: pantau.tinggi_lokal,
            tinggi_jaringan: pantau.tinggi_jaringan,
            rekan: pantau.rekan,
            folder_data: self.folder.display().to_string(),
            pesan_galat: pantau.galat.as_ref().map(|g| g.0.clone()),
            detail_galat: pantau.galat.as_ref().map(|g| g.1.clone()),
        }
    }

    /// Menyalakan node. Node lain yang sudah menjawab di RPC lokal dipakai apa adanya.
    pub(crate) fn nyalakan_node(self: &Arc<Self>) -> Result<(), String> {
        if kunci(&self.node).is_some() || self.node_luar.load(Ordering::SeqCst) {
            return Ok(());
        }
        kunci(&self.pantau).galat = None;
        if tinggi_di(Cli::connect(RPC_LOKAL, Network::Regtest).ok().as_ref()).is_some() {
            self.node_luar.store(true, Ordering::SeqCst);
            self.catat(
                "aplikasi",
                "node lain sudah berjalan di 127.0.0.1:28801; dipakai apa adanya",
            );
            return Ok(());
        }
        std::fs::create_dir_all(&self.folder).map_err(|e| {
            format!(
                "folder data {} tidak bisa dibuat: {e}",
                self.folder.display()
            )
        })?;
        let mut child = perintah(&self.bin_node)
            .arg("--regtest")
            .arg("--datadir")
            .arg(&self.folder)
            .arg("--seed-resmi")
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                format!(
                    "node tidak bisa dijalankan ({}): {e}",
                    self.bin_node.display()
                )
            })?;
        {
            let mut pantau = kunci(&self.pantau);
            pantau.rekan = 0;
            pantau.tinggi_lokal = None;
        }
        if let Some(out) = child.stdout.take() {
            self.baca(out, "node", false);
        }
        if let Some(err) = child.stderr.take() {
            self.baca(err, "node", true);
        }
        *kunci(&self.node) = Some(child);
        self.catat("aplikasi", "node dinyalakan (--seed-resmi)");
        Ok(())
    }

    /// Utas pembaca keluaran proses anak ke log; `rekan` menghitung rekan dari baris node.
    fn baca<R: Read + Send + 'static>(
        self: &Arc<Self>,
        sumber_io: R,
        sumber: &'static str,
        rekan: bool,
    ) {
        let mesin = Arc::clone(self);
        thread::spawn(move || {
            for baris in BufReader::new(sumber_io).lines() {
                let Ok(baris) = baris else { break };
                let baris = bersihkan_baris(&baris);
                if rekan && let Some(beda) = rekan_dari_baris(&baris) {
                    let mut pantau = kunci(&mesin.pantau);
                    pantau.rekan = if beda > 0 {
                        pantau.rekan.saturating_add(1)
                    } else {
                        pantau.rekan.saturating_sub(1)
                    };
                }
                if sumber == "penambang"
                    && let Some((tinggi, id_blok)) = blok_dari_baris(&baris)
                {
                    let mut tambang = kunci(&mesin.tambang);
                    tambang.blok_sesi = tambang.blok_sesi.saturating_add(1);
                    tambang.blok.insert(
                        0,
                        BlokDitemukan {
                            tinggi,
                            id_blok,
                            waktu: sekarang_ms(),
                        },
                    );
                    tambang.blok.truncate(MAKS_BLOK);
                }
                mesin.catat(sumber, &baris);
            }
        });
    }

    /// Menghentikan node yang dinyalakan aplikasi: SIGTERM (Unix) lalu tunggu, paksa sesudah 30 detik. Di Windows langsung
    /// dihentikan (penyimpanan chain redb dan `peers.json` aman terhadap penghentian mendadak).
    pub(crate) fn hentikan_node(&self) {
        self.hentikan_tambang();
        self.node_luar.store(false, Ordering::SeqCst);
        let Some(mut child) = kunci(&self.node).take() else {
            return;
        };
        if cfg!(unix) {
            let _ = Command::new("kill")
                .arg("-TERM")
                .arg(child.id().to_string())
                .status();
            let batas = Instant::now() + TUNGGU_BERHENTI;
            while Instant::now() < batas {
                if matches!(child.try_wait(), Ok(Some(_))) {
                    break;
                }
                thread::sleep(Duration::from_millis(200));
            }
        }
        if !matches!(child.try_wait(), Ok(Some(_))) {
            let _ = child.kill();
        }
        let _ = child.wait();
        let mut pantau = kunci(&self.pantau);
        pantau.rekan = 0;
        pantau.tinggi_lokal = None;
        drop(pantau);
        self.catat("aplikasi", "node dihentikan");
    }

    pub(crate) fn ada_proses(&self) -> bool {
        kunci(&self.node).is_some() || kunci(&self.tambang).status != "berhenti"
    }

    pub(crate) fn keadaan_tambang(&self) -> KeadaanTambang {
        let t = kunci(&self.tambang);
        KeadaanTambang {
            status: t.status,
            alamat: t.alamat.clone(),
            jeda_detik: t.jeda,
            sisa_jeda_detik: t.sisa,
            blok_sesi_ini: t.blok_sesi,
            blok_terakhir: t.blok.clone(),
            pesan_galat: t.galat.as_ref().map(|g| g.0.clone()),
            detail_galat: t.galat.as_ref().map(|g| g.1.clone()),
        }
    }

    /// Mulai menambang ke `alamat` dengan jeda `jeda` detik; hanya bila node sinkron.
    pub(crate) fn mulai_tambang(self: &Arc<Self>, alamat: &str, jeda: u64) -> Result<(), String> {
        if !JEDA_SAH.contains(&jeda) {
            return self.tolak_tambang("jeda harus 60, 120, atau 300 detik");
        }
        let (hasil, payload) = periksa_alamat(alamat);
        let Some((jenis, payload)) = payload.filter(|_| hasil.sah) else {
            return self
                .tolak_tambang("alamat hadiah bukan alamat jaringan demo (rzvrg1...) yang sah");
        };
        if self.keadaan_node().status != "sinkron" {
            return self.tolak_tambang("node belum sinkron dengan jaringan demo");
        }
        let id = self
            .putaran
            .fetch_add(1, Ordering::SeqCst)
            .saturating_add(1);
        {
            let mut t = kunci(&self.tambang);
            t.status = "menambang";
            t.alamat = Some(alamat.trim().to_owned());
            t.jeda = jeda;
            t.sisa = None;
            t.galat = None;
        }
        self.catat(
            "aplikasi",
            &format!("menambang: satu blok setiap {jeda} detik"),
        );
        let mesin = Arc::clone(self);
        thread::spawn(move || mesin.putar_tambang(id, jenis, &payload, jeda));
        Ok(())
    }

    fn tolak_tambang(&self, pesan: &str) -> Result<(), String> {
        kunci(&self.tambang).galat = Some((pesan.to_owned(), String::new()));
        Err(pesan.to_owned())
    }

    fn masih(&self, id: u64) -> bool {
        self.putaran.load(Ordering::SeqCst) == id
    }

    /// Utas penambang: satu blok, lalu jeda, sampai penambangan dihentikan.
    fn putar_tambang(self: Arc<Self>, id: u64, jenis: u8, payload: &str, jeda: u64) {
        while self.masih(id) {
            {
                let mut t = kunci(&self.tambang);
                t.status = "menambang";
                t.sisa = None;
            }
            match perintah(&self.bin_miner)
                .arg("--payout")
                .arg(payload)
                .arg("--payout-kind")
                .arg(jenis.to_string())
                .arg("--blocks")
                .arg("1")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    if let Some(out) = child.stdout.take() {
                        self.baca(out, "penambang", false);
                    }
                    if let Some(err) = child.stderr.take() {
                        self.baca(err, "penambang", false);
                    }
                    *kunci(&self.miner) = Some(child);
                    loop {
                        let selesai = kunci(&self.miner)
                            .as_mut()
                            .is_none_or(|c| matches!(c.try_wait(), Ok(Some(_)) | Err(_)));
                        if selesai || !self.masih(id) {
                            break;
                        }
                        thread::sleep(Duration::from_millis(200));
                    }
                    if let Some(mut child) = kunci(&self.miner).take() {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                }
                Err(e) => {
                    let pesan = format!(
                        "penambang tidak bisa dijalankan ({}): {e}",
                        self.bin_miner.display()
                    );
                    self.catat("aplikasi", &pesan);
                    let mut t = kunci(&self.tambang);
                    t.galat = Some((pesan, String::new()));
                    t.status = "berhenti";
                    drop(t);
                    self.putaran.fetch_add(1, Ordering::SeqCst);
                    return;
                }
            }
            let mulai = Instant::now();
            while self.masih(id) {
                let lewat = mulai.elapsed().as_secs();
                if lewat >= jeda {
                    break;
                }
                {
                    let mut t = kunci(&self.tambang);
                    t.status = "menunggu";
                    t.sisa = Some(jeda.saturating_sub(lewat));
                }
                thread::sleep(Duration::from_millis(200));
            }
        }
    }

    /// Menghentikan penambangan (miner yang sedang berjalan dihentikan).
    pub(crate) fn hentikan_tambang(&self) {
        self.putaran.fetch_add(1, Ordering::SeqCst);
        if let Some(mut child) = kunci(&self.miner).take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let mut t = kunci(&self.tambang);
        if t.status != "berhenti" {
            t.status = "berhenti";
            t.sisa = None;
            drop(t);
            self.catat("aplikasi", "menambang dihentikan");
        }
    }

    pub(crate) fn hentikan_semua(&self) {
        self.hentikan_tambang();
        self.hentikan_node();
    }

    pub(crate) fn tandai_akan_tutup(&self) {
        self.akan_tutup.store(true, Ordering::SeqCst);
    }

    pub(crate) fn akan_tutup(&self) -> bool {
        self.akan_tutup.load(Ordering::SeqCst)
    }

    pub(crate) fn muat_pengaturan(&self) -> Pengaturan {
        std::fs::read_to_string(self.folder.join(BERKAS_PENGATURAN))
            .map(|t| Pengaturan::dari_json(&t))
            .unwrap_or_default()
    }

    pub(crate) fn simpan_pengaturan(&self, pengaturan: Pengaturan) -> Result<(), String> {
        let rapi = pengaturan.rapikan();
        let teks = serde_json::to_string_pretty(&rapi).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&self.folder).map_err(|e| e.to_string())?;
        let akhir = self.folder.join(BERKAS_PENGATURAN);
        let sementara = self.folder.join(format!("{BERKAS_PENGATURAN}.tmp"));
        std::fs::write(&sementara, teks).map_err(|e| e.to_string())?;
        std::fs::rename(&sementara, &akhir).map_err(|e| e.to_string())
    }

    /// Versi `zevrigo-node` (keluaran `--version`), atau teks kosong.
    pub(crate) fn versi_node(&self) -> String {
        perintah(&self.bin_node)
            .arg("--version")
            .stdin(Stdio::null())
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests;
