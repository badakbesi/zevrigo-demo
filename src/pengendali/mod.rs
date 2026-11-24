//! Pengendali jarak jauh Zevrigo Node (DEC-45, RENCANA_T17 bagian K1 dan K2): node `--seed-resmi` dan penambang dijalankan
//! sendiri lewat mesin yang sama dengan aplikasi desktop ([`zevrigo_node_mesin::Mesin`]), lalu dikendalikan lewat HTTP dari
//! aplikasi Zevrigo Node di Android. Tidak memakai Docker socket dan tidak menyentuh layanan lain.
//!
//! - Satu permintaan JSON-RPC 2.0 per `POST`, hanya metode di [`METODE_PENGENDALI`] (sama dengan tombol aplikasi). Tidak ada
//!   perintah bebas, shell, atau akses berkas. Parameter berupa objek.
//! - Token akses wajib: `Authorization: Bearer <token>` (32 sampai 256 karakter). Pengendali hanya menyimpan SHA-256 token di
//!   berkas `--token-sha256` (format keluaran `sha256sum`), dibaca ulang setiap permintaan sehingga token bisa diganti atau
//!   dicabut tanpa memulai ulang; dibandingkan tanpa berhenti di byte pertama yang beda. Di depan pengendali ada Cloudflare
//!   Access (service token, keputusan pencipta 24 November 2026); pengendali tetap memeriksa tokennya sendiri.
//! - Batas laju per pengunjung dan bersama ([`crate::limit`]) sebelum token diperiksa; token salah berulang dijawab `429`
//!   (ember kedua, 1 per detik, kapasitas 10). Alamat pengunjung dari kepala Cloudflare hanya untuk koneksi dari proxy
//!   `--trust-proxy-host` ([`faucet_client_ip`]); alamat tidak disimpan dan tidak dicatat.
//! - Perintah yang mengubah keadaan dicatat di log program dan di log mesin (`perintah jarak jauh: ...`), tanpa token.
//! - Saat menerima SIGINT atau SIGTERM, server berhenti lalu penambang dan node dihentikan.

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{ConnectInfo, State};
use axum::http::{Extensions, HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use zevrigo_node_mesin::{Mesin, Pengaturan, bin_sidecar, periksa_alamat};
use zevrigo_rpc::error::{
    INTERNAL_ERROR, INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND, PARSE_ERROR,
};

use crate::cli::PengendaliConfig;
use crate::faucet::faucet_client_ip;
use crate::gateway::{DemoError, shutdown_signal};
use crate::limit::{Limits, Rate, RateLimiter};

/// Metode yang dilayani: kontrak `Backend` aplikasi Zevrigo Node tanpa urusan jendela, papan klip, dan tautan.
pub const METODE_PENGENDALI: [&str; 11] = [
    "info",
    "keadaan_node",
    "keadaan_tambang",
    "log_terbaru",
    "muat_pengaturan",
    "simpan_pengaturan",
    "periksa_alamat",
    "nyalakan_node",
    "hentikan_node",
    "mulai_tambang",
    "berhenti_tambang",
];

/// Metode yang mengubah keadaan; dicatat di log program dan log mesin.
const METODE_KENDALI: [&str; 5] = [
    "simpan_pengaturan",
    "nyalakan_node",
    "hentikan_node",
    "mulai_tambang",
    "berhenti_tambang",
];

/// Ukuran badan permintaan paling besar.
pub const MAX_BODY_BYTES: usize = 16_384;

/// Panjang token paling pendek (karakter).
pub const TOKEN_MIN: usize = 32;

/// Panjang token paling panjang (karakter).
pub const TOKEN_MAX: usize = 256;

/// Kode galat JSON-RPC untuk perintah yang ditolak mesin, misalnya node belum sinkron atau program node tidak ada.
pub const GALAT_PERINTAH: i32 = -32_000;

/// Batas laju permintaan dengan token salah: per pengunjung 1 per detik (kapasitas 10), semua pengunjung 20 per detik.
const BATAS_GAGAL: Limits = Limits {
    client: Rate {
        per_second: 1,
        burst: 10,
    },
    global: Rate {
        per_second: 20,
        burst: 200,
    },
};

/// Batas waktu menunggu tugas selesai saat pengendali berhenti.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Selang me-resolve ulang nama host proxy tepercaya.
const RESOLVE_INTERVAL: Duration = Duration::from_secs(60);

/// SHA-256 dari isi berkas token: kata pertama berupa tepat 64 huruf hex (format keluaran `sha256sum`).
#[must_use]
pub fn baca_hash_token(teks: &str) -> Option<[u8; 32]> {
    let kata = teks.split_whitespace().next()?;
    if kata.len() != 64 || !kata.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let mut hasil = [0_u8; 32];
    for (i, slot) in hasil.iter_mut().enumerate() {
        let awal = i.checked_mul(2)?;
        let pasangan = kata.get(awal..awal.checked_add(2)?)?;
        *slot = u8::from_str_radix(pasangan, 16).ok()?;
    }
    Some(hasil)
}

/// Token dari kepala `Authorization: Bearer <token>`: [`TOKEN_MIN`] sampai [`TOKEN_MAX`] karakter ASCII terlihat.
#[must_use]
pub fn token_dari_kepala(headers: &HeaderMap) -> Option<&str> {
    let nilai = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (skema, token) = nilai.trim().split_once(' ')?;
    let token = token.trim();
    let sah = skema.eq_ignore_ascii_case("bearer")
        && (TOKEN_MIN..=TOKEN_MAX).contains(&token.len())
        && token.bytes().all(|b| b.is_ascii_graphic());
    sah.then_some(token)
}

/// `true` bila SHA-256 `token` sama dengan `hash`. Semua byte dibandingkan (tanpa berhenti di byte pertama yang beda).
#[must_use]
pub fn token_cocok(token: &str, hash: &[u8; 32]) -> bool {
    let dihitung = Sha256::digest(token.as_bytes());
    let beda = dihitung
        .iter()
        .zip(hash.iter())
        .fold(0_u8, |acc, (a, b)| acc | (a ^ b));
    std::hint::black_box(beda) == 0 && dihitung.len() == hash.len()
}

/// Satu permintaan JSON-RPC ke pengendali yang lolos pemeriksaan bentuk.
#[derive(Clone, Debug, PartialEq)]
pub struct Perintah {
    /// `id` permintaan (angka atau teks), dikembalikan apa adanya.
    pub id: Value,
    /// Nama metode, salah satu dari [`METODE_PENGENDALI`].
    pub metode: String,
    /// Parameter; tanpa parameter atau `null` menjadi objek kosong.
    pub params: Map<String, Value>,
}

/// Jawaban galat JSON-RPC 2.0.
fn error_response(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "error": {"code": code, "message": message}, "id": id})
}

/// Memeriksa badan satu permintaan JSON-RPC 2.0.
///
/// # Errors
///
/// Jawaban galat JSON-RPC: `-32700` untuk JSON rusak; `-32600` untuk batch, badan yang bukan objek, `id` yang bukan angka
/// atau teks, `jsonrpc` yang bukan `"2.0"`, atau `method` yang bukan teks; `-32601` untuk metode di luar
/// [`METODE_PENGENDALI`]; `-32602` untuk parameter yang bukan objek.
pub fn parse_perintah(body: &[u8]) -> Result<Perintah, Value> {
    let request: Value = serde_json::from_slice(body)
        .map_err(|_| error_response(Value::Null, PARSE_ERROR, "Parse error"))?;
    let batch = request.is_array();
    let Value::Object(mut object) = request else {
        let message = if batch {
            "batch tidak dilayani pengendali"
        } else {
            "Invalid request"
        };
        return Err(error_response(Value::Null, INVALID_REQUEST, message));
    };
    let id = match object.remove("id") {
        Some(id @ (Value::Number(_) | Value::String(_))) => id,
        _ => {
            return Err(error_response(
                Value::Null,
                INVALID_REQUEST,
                "id wajib berupa angka atau teks",
            ));
        }
    };
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(error_response(id, INVALID_REQUEST, "Invalid request"));
    }
    let metode = match object.remove("method") {
        Some(Value::String(metode)) => metode,
        _ => return Err(error_response(id, INVALID_REQUEST, "Invalid request")),
    };
    if !METODE_PENGENDALI.contains(&metode.as_str()) {
        return Err(error_response(
            id,
            METHOD_NOT_FOUND,
            "metode tidak tersedia di pengendali",
        ));
    }
    let params = match object.remove("params") {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(params)) => params,
        Some(_) => {
            return Err(error_response(
                id,
                INVALID_PARAMS,
                "params wajib berupa objek",
            ));
        }
    };
    Ok(Perintah { id, metode, params })
}

/// Pilihan router pengendali.
pub struct OpsiPengendali {
    /// Berkas berisi SHA-256 token akses (dibaca ulang setiap permintaan).
    pub token_file: PathBuf,
    /// Batas laju per pengunjung (`None` = tanpa batas).
    pub limits: Option<Limits>,
    /// Alamat proxy tepercaya, diperbarui dari luar.
    pub trusted: Arc<RwLock<Vec<IpAddr>>>,
    /// Versi program node (jawaban `info`).
    pub versi_node: String,
}

/// Isi router.
struct App {
    mesin: Arc<Mesin>,
    token_file: PathBuf,
    limiter: Option<Mutex<RateLimiter>>,
    gagal: Mutex<RateLimiter>,
    trusted: Arc<RwLock<Vec<IpAddr>>>,
    versi_node: String,
    started: Instant,
}

/// Hasil satu metode: nilai `result`, atau `(kode, pesan)` galat JSON-RPC.
type Hasil = Result<Value, (i32, String)>;

impl App {
    fn now_ms(&self) -> u64 {
        u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    /// Alamat pengunjung untuk batas laju (tidak disimpan dan tidak dicatat).
    fn visitor(&self, extensions: &Extensions, headers: &HeaderMap) -> Option<IpAddr> {
        let connect = extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|info| info.0.ip());
        let trusted = self
            .trusted
            .read()
            .map(|list| list.clone())
            .unwrap_or_default();
        faucet_client_ip(connect, headers, &trusted)
    }

    fn allow(&self, visitor: Option<IpAddr>) -> bool {
        let Some(limiter) = &self.limiter else {
            return true;
        };
        let now_ms = self.now_ms();
        limiter
            .lock()
            .map(|mut limiter| limiter.allow(visitor, now_ms))
            .unwrap_or(false)
    }

    fn allow_gagal(&self, visitor: Option<IpAddr>) -> bool {
        let now_ms = self.now_ms();
        self.gagal
            .lock()
            .map(|mut limiter| limiter.allow(visitor, now_ms))
            .unwrap_or(false)
    }

    /// `true` bila kepala `Authorization` membawa token yang SHA-256-nya ada di berkas token.
    fn token_sah(&self, headers: &HeaderMap) -> bool {
        let Some(token) = token_dari_kepala(headers) else {
            return false;
        };
        let hash = std::fs::read_to_string(&self.token_file)
            .ok()
            .as_deref()
            .and_then(baca_hash_token);
        match hash {
            Some(hash) => token_cocok(token, &hash),
            None => {
                tracing::warn!("berkas token tidak terbaca atau tidak sah; permintaan ditolak");
                false
            }
        }
    }
}

/// Router axum pengendali: setiap jalur dilayani [`handle`]. Batas per pengunjung butuh
/// `into_make_service_with_connect_info::<SocketAddr>()`.
pub fn router(mesin: Arc<Mesin>, opsi: OpsiPengendali) -> Router {
    let app = App {
        mesin,
        token_file: opsi.token_file,
        limiter: opsi
            .limits
            .map(|limits| Mutex::new(RateLimiter::new(limits))),
        gagal: Mutex::new(RateLimiter::new(BATAS_GAGAL)),
        trusted: opsi.trusted,
        versi_node: opsi.versi_node,
        started: Instant::now(),
    };
    Router::new().fallback(handle).with_state(Arc::new(app))
}

/// Jawaban JSON dengan `no-store` dan `nosniff`.
fn json_reply(status: StatusCode, body: Vec<u8>) -> Response {
    (
        status,
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-store"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        body,
    )
        .into_response()
}

/// `429 {"error": "rate_limited"}` dengan `Retry-After: 1`.
fn rate_limited() -> Response {
    let mut response = json_reply(
        StatusCode::TOO_MANY_REQUESTS,
        b"{\"error\":\"rate_limited\"}".to_vec(),
    );
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
    response
}

/// `401 {"error": "unauthorized"}` dengan `WWW-Authenticate: Bearer`.
fn unauthorized() -> Response {
    let mut response = json_reply(
        StatusCode::UNAUTHORIZED,
        b"{\"error\":\"unauthorized\"}".to_vec(),
    );
    response
        .headers_mut()
        .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
    response
}

/// Satu permintaan HTTP: batas laju, lalu metode `POST`, lalu token, lalu badan paling banyak [`MAX_BODY_BYTES`], lalu
/// [`parse_perintah`] dan perintahnya. Jawaban JSON-RPC (hasil atau galat) selalu berstatus `200`.
async fn handle(
    State(app): State<Arc<App>>,
    method: Method,
    headers: HeaderMap,
    extensions: Extensions,
    body: Body,
) -> Response {
    let visitor = app.visitor(&extensions, &headers);
    if !app.allow(visitor) {
        return rate_limited();
    }
    if method != Method::POST {
        let mut response = json_reply(
            StatusCode::METHOD_NOT_ALLOWED,
            b"{\"error\":\"method_not_allowed\"}".to_vec(),
        );
        response
            .headers_mut()
            .insert(header::ALLOW, HeaderValue::from_static("POST"));
        return response;
    }
    if !app.token_sah(&headers) {
        return if app.allow_gagal(visitor) {
            unauthorized()
        } else {
            rate_limited()
        };
    }
    let Ok(bytes) = to_bytes(body, MAX_BODY_BYTES).await else {
        return json_reply(
            StatusCode::PAYLOAD_TOO_LARGE,
            b"{\"error\":\"payload_too_large\"}".to_vec(),
        );
    };
    let answer = match parse_perintah(&bytes) {
        Ok(perintah) => jalankan(&app, perintah).await,
        Err(answer) => answer,
    };
    json_reply(StatusCode::OK, answer.to_string().into_bytes())
}

/// Menjalankan `job` di utas sendiri tanpa konteks runtime (mesin memakai runtime tokio sendiri untuk klien RPC).
async fn blocking<T, F>(job: F) -> Result<T, (i32, String)>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let berhenti = || (INTERNAL_ERROR, "pekerjaan berhenti tak terduga".to_owned());
    tokio::task::spawn_blocking(move || std::thread::spawn(job).join())
        .await
        .map_err(|_| berhenti())?
        .map_err(|_| berhenti())
}

fn internal(error: &serde_json::Error) -> (i32, String) {
    (INTERNAL_ERROR, error.to_string())
}

fn ditolak(pesan: String) -> (i32, String) {
    (GALAT_PERINTAH, pesan)
}

fn param_teks(params: &Map<String, Value>, nama: &str) -> Result<String, (i32, String)> {
    params
        .get(nama)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| (INVALID_PARAMS, format!("{nama} wajib berupa teks")))
}

/// Teks pendek dan aman untuk log (huruf dan angka ASCII saja, paling banyak 100).
fn untuk_log(nilai: Option<&Value>) -> String {
    let teks = match nilai {
        Some(Value::String(teks)) => teks.clone(),
        Some(Value::Number(angka)) => angka.to_string(),
        _ => "-".to_owned(),
    };
    teks.chars()
        .filter(char::is_ascii_alphanumeric)
        .take(100)
        .collect()
}

/// Mencatat perintah kendali di log program dan log mesin (tanpa token dan tanpa alamat pengunjung).
fn catat_kendali(app: &App, metode: &str, params: &Map<String, Value>) {
    let rincian = match metode {
        "mulai_tambang" => format!(
            " (alamat {}, jeda {} detik)",
            untuk_log(params.get("alamat")),
            untuk_log(params.get("jeda"))
        ),
        "simpan_pengaturan" => {
            let pengaturan = params.get("pengaturan");
            format!(
                " (alamat hadiah {}, jeda {} detik)",
                untuk_log(pengaturan.and_then(|p| p.get("alamatHadiah"))),
                untuk_log(pengaturan.and_then(|p| p.get("jedaDetik")))
            )
        }
        _ => String::new(),
    };
    tracing::info!(perintah = metode, rincian = %rincian, "perintah kendali");
    app.mesin.catat(
        "aplikasi",
        &format!("perintah jarak jauh: {metode}{rincian}"),
    );
}

/// Menjalankan satu perintah dan menyusun jawaban JSON-RPC.
async fn jalankan(app: &App, perintah: Perintah) -> Value {
    let Perintah { id, metode, params } = perintah;
    if METODE_KENDALI.contains(&metode.as_str()) {
        catat_kendali(app, &metode, &params);
    }
    match hasil(app, &metode, &params).await {
        Ok(result) => json!({"jsonrpc": "2.0", "result": result, "id": id}),
        Err((code, message)) => error_response(id, code, &message),
    }
}

async fn hasil(app: &App, metode: &str, params: &Map<String, Value>) -> Hasil {
    let mesin = Arc::clone(&app.mesin);
    let nilai = match metode {
        "info" => json!({"versi": env!("CARGO_PKG_VERSION"), "versiNode": app.versi_node}),
        "keadaan_node" => serde_json::to_value(mesin.keadaan_node()).map_err(|e| internal(&e))?,
        "keadaan_tambang" => {
            serde_json::to_value(mesin.keadaan_tambang()).map_err(|e| internal(&e))?
        }
        "log_terbaru" => {
            let sejak = match params.get("sejak") {
                None | Some(Value::Null) => None,
                Some(nilai) => Some(nilai.as_u64().ok_or_else(|| {
                    (
                        INVALID_PARAMS,
                        "sejak wajib berupa milidetik atau null".to_owned(),
                    )
                })?),
            };
            serde_json::to_value(mesin.log_sejak(sejak)).map_err(|e| internal(&e))?
        }
        "muat_pengaturan" => {
            serde_json::to_value(mesin.muat_pengaturan()).map_err(|e| internal(&e))?
        }
        "periksa_alamat" => {
            let alamat = param_teks(params, "alamat")?;
            serde_json::to_value(periksa_alamat(&alamat).0).map_err(|e| internal(&e))?
        }
        "simpan_pengaturan" => {
            let pengaturan: Pengaturan = params
                .get("pengaturan")
                .filter(|nilai| nilai.is_object())
                .cloned()
                .and_then(|nilai| serde_json::from_value(nilai).ok())
                .ok_or_else(|| {
                    (
                        INVALID_PARAMS,
                        "pengaturan wajib berupa objek pengaturan".to_owned(),
                    )
                })?;
            blocking(move || mesin.simpan_pengaturan(pengaturan))
                .await?
                .map_err(ditolak)?;
            Value::Null
        }
        "nyalakan_node" => {
            blocking(move || mesin.nyalakan_node())
                .await?
                .map_err(ditolak)?;
            Value::Null
        }
        "hentikan_node" => {
            blocking(move || mesin.hentikan_node()).await?;
            Value::Null
        }
        "mulai_tambang" => {
            let alamat = param_teks(params, "alamat")?;
            let jeda = params
                .get("jeda")
                .and_then(Value::as_u64)
                .ok_or_else(|| (INVALID_PARAMS, "jeda wajib berupa detik".to_owned()))?;
            blocking(move || mesin.mulai_tambang(&alamat, jeda))
                .await?
                .map_err(ditolak)?;
            Value::Null
        }
        "berhenti_tambang" => {
            blocking(move || mesin.hentikan_tambang()).await?;
            Value::Null
        }
        _ => {
            return Err((
                METHOD_NOT_FOUND,
                "metode tidak tersedia di pengendali".to_owned(),
            ));
        }
    };
    Ok(nilai)
}

/// Mengganti daftar alamat proxy tepercaya.
fn ganti_trusted(trusted: &RwLock<Vec<IpAddr>>, alamat: Vec<IpAddr>) {
    if let Ok(mut slot) = trusted.write() {
        *slot = alamat;
    }
}

async fn serve(config: &PengendaliConfig) -> Result<(), DemoError> {
    let shutdown = shutdown_signal()?;
    let isi = std::fs::read_to_string(&config.token_file).unwrap_or_default();
    if baca_hash_token(&isi).is_none() {
        return Err(DemoError::Pengendali(format!(
            "berkas {} tidak berisi SHA-256 token (64 huruf hex)",
            config.token_file.display()
        )));
    }
    let mesin = Arc::new(Mesin::baru(
        config.datadir.clone(),
        bin_sidecar("zevrigo-node", "ZEVRIGO_NODE_BIN"),
        bin_sidecar("zevrigo-miner", "ZEVRIGO_MINER_BIN"),
    ));
    mesin.mulai_pemantau();
    let versi_node = {
        let mesin = Arc::clone(&mesin);
        blocking(move || mesin.versi_node())
            .await
            .unwrap_or_default()
    };
    if config.nyalakan_node {
        let m = Arc::clone(&mesin);
        match blocking(move || m.nyalakan_node()).await {
            Ok(Ok(())) => tracing::info!("node dinyalakan saat mulai"),
            Ok(Err(error)) | Err((_, error)) => {
                tracing::warn!(%error, "node tidak bisa dinyalakan saat mulai");
            }
        }
    }
    let trusted = Arc::new(RwLock::new(Vec::new()));
    if let Some(host) = config.trust_proxy_host.clone() {
        let trusted = Arc::clone(&trusted);
        tokio::spawn(async move {
            loop {
                if let Ok(found) = tokio::net::lookup_host((host.as_str(), 0)).await {
                    ganti_trusted(&trusted, found.map(|address| address.ip()).collect());
                }
                tokio::time::sleep(RESOLVE_INTERVAL).await;
            }
        });
    }
    let listener =
        TcpListener::bind(SocketAddr::from((config.http_bind, config.http_port))).await?;
    let local = listener.local_addr()?;
    tracing::info!(
        %local,
        datadir = %config.datadir.display(),
        rate = config.rate,
        node = %versi_node,
        "pengendali berjalan"
    );
    println!("http: http://{local}");
    let app = router(
        Arc::clone(&mesin),
        OpsiPengendali {
            token_file: config.token_file.clone(),
            limits: Limits::from_rate(config.rate),
            trusted,
            versi_node,
        },
    );
    let served = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await;
    tracing::info!("pengendali berhenti; penambang dan node dihentikan");
    let m = Arc::clone(&mesin);
    // Galat di sini berarti utas penghenti panik; proses tetap keluar.
    let _ = blocking(move || m.hentikan_semua()).await;
    Ok(served?)
}

/// Menjalankan pengendali (mode `pengendali`) sampai menerima SIGINT atau SIGTERM, lalu menghentikan penambang dan node.
///
/// # Errors
///
/// [`DemoError`] jika runtime, sinyal, berkas token, atau port HTTP gagal.
pub fn run_pengendali(config: &PengendaliConfig) -> Result<(), DemoError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let result = runtime.block_on(serve(config));
    runtime.shutdown_timeout(SHUTDOWN_TIMEOUT);
    result
}

#[cfg(test)]
mod tests;
