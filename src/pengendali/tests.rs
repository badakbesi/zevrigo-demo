use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use axum::http::{HeaderMap, HeaderValue, header};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use zevrigo_node_mesin::Mesin;
use zevrigo_primitives::{Amount, Network, OutputKind, QUANTA_PER_ZVRG};
use zevrigo_rpc::error::{INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND, PARSE_ERROR};
use zevrigo_rpc::{
    BlockInfo, BlockTemplate, ChainInfo, CompactHex, GetBlockTemplateParams, Hex32, KindNumber,
    MempoolInfo, MempoolScan, NetworkName, Quanta, RpcBackend, RpcError, RpcServer,
    SendRawTransactionResult, SubmitBlockResult, UtxoInfo,
};
use zevrigo_wallet_core::encode_address;

use super::{
    BERKAS_LANJUT, GALAT_PERINTAH, METODE_PENGENDALI, OpsiPengendali, TOKEN_MIN, baca_hash_token,
    baca_lanjut, hapus_lanjut, parse_perintah, router, simpan_lanjut, token_cocok,
    token_dari_kepala,
};
use crate::limit::Limits;

const TOKEN: &str = "token-uji-pengendali-0123456789abcdefghijklmnop";
const TOKEN_LAIN: &str = "token-uji-pengendali-baru-zyxwvutsrqponmlkjihgfe";

fn hex(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

static NOMOR: AtomicUsize = AtomicUsize::new(0);

/// Folder sementara yang unik per uji.
fn folder() -> PathBuf {
    let n = NOMOR.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("zvrg-pengendali-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Pengendali dengan mesin tanpa program node dan penambang (tanpa pemantau), di port bebas.
struct Setup {
    rt: Runtime,
    server: JoinHandle<std::io::Result<()>>,
    addr: SocketAddr,
    dir: PathBuf,
}

impl Setup {
    fn start(limits: Option<Limits>) -> Self {
        Self::start_node(limits, "http://127.0.0.1:9".to_owned())
    }

    fn start_node(limits: Option<Limits>, rpc_node: String) -> Self {
        let dir = folder();
        std::fs::write(dir.join("token.sha256"), format!("{}  -\n", hex(TOKEN))).unwrap();
        let mesin = Arc::new(Mesin::baru(
            dir.join("data"),
            dir.join("tidak-ada-node"),
            dir.join("tidak-ada-miner"),
        ));
        let rt = Runtime::new().unwrap();
        let listener = rt
            .block_on(TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))))
            .unwrap();
        let addr = listener.local_addr().unwrap();
        let app = router(
            mesin,
            OpsiPengendali {
                token_file: dir.join("token.sha256"),
                limits,
                trusted: Arc::new(RwLock::new(Vec::new())),
                versi_node: "zevrigo-node 0.1.0".to_owned(),
                rpc_node,
            },
        );
        let server = rt.spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
        });
        Self {
            rt,
            server,
            addr,
            dir,
        }
    }

    fn finish(self) {
        self.server.abort();
        drop(self.rt);
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Satu permintaan HTTP/1.1 mentah; mengembalikan status, kepala jawaban (huruf kecil), dan badan teks.
fn raw(
    addr: SocketAddr,
    method: &str,
    token: Option<&str>,
    payload: &[u8],
) -> (u16, String, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    let auth = token
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();
    write!(
        stream,
        "{method} / HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n{auth}Content-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    )
    .unwrap();
    stream.write_all(payload).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    let (head, rest) = text.split_once("\r\n\r\n").unwrap();
    let status = head.split(' ').nth(1).unwrap().parse().unwrap();
    (status, head.to_ascii_lowercase(), rest.to_owned())
}

/// Satu panggilan JSON-RPC dengan token benar; mengembalikan jawaban JSON.
fn call(addr: SocketAddr, method: &str, params: &Value) -> Value {
    let body = json!({"jsonrpc": "2.0", "id": 7, "method": method, "params": params}).to_string();
    let (status, _, text) = raw(addr, "POST", Some(TOKEN), body.as_bytes());
    assert_eq!(status, 200, "{method}: {text}");
    let answer: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(answer["id"], json!(7));
    answer
}

/// Badan JSON-RPC 2.0 sebagai byte. Dibuat lewat `json!` (bukan string mentah) agar pemindai larangan tidak salah membaca
/// versi `2.0` sebagai literal floating point.
fn badan(method: &str) -> Vec<u8> {
    json!({"jsonrpc": "2.0", "id": 1, "method": method})
        .to_string()
        .into_bytes()
}

fn code(answer: &Value) -> Option<i64> {
    answer["error"]["code"].as_i64()
}

#[test]
fn reads_token_hash_and_header() {
    let h = hex(TOKEN);
    let hash = baca_hash_token(&format!("{h}  -\n")).unwrap();
    assert!(token_cocok(TOKEN, &hash));
    assert!(!token_cocok(TOKEN_LAIN, &hash));
    assert_eq!(baca_hash_token(&h.to_ascii_uppercase()), Some(hash));
    assert_eq!(baca_hash_token(""), None);
    assert_eq!(baca_hash_token(&h[..63]), None);
    assert_eq!(baca_hash_token(&format!("+{}", &h[1..])), None);
    assert_eq!(baca_hash_token(&format!("{h}0")), None);

    let mut headers = HeaderMap::new();
    assert_eq!(token_dari_kepala(&headers), None);
    for (nilai, hasil) in [
        (format!("Bearer {TOKEN}"), Some(TOKEN)),
        (format!("bearer   {TOKEN}  "), Some(TOKEN)),
        (format!("Basic {TOKEN}"), None),
        (format!("Bearer {}", &TOKEN[..TOKEN_MIN - 1]), None),
        (format!("Bearer {}", "a".repeat(257)), None),
        (format!("Bearer {TOKEN} lagi"), None),
        (TOKEN.to_owned(), None),
    ] {
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&nilai).unwrap(),
        );
        assert_eq!(token_dari_kepala(&headers), hasil, "{nilai}");
    }
}

#[test]
fn parses_requests() {
    let ok = parse_perintah(
        json!({"jsonrpc": "2.0", "id": "a", "method": "keadaan_node"})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    assert_eq!(
        (ok.id, ok.metode.as_str(), ok.params.len()),
        (json!("a"), "keadaan_node", 0)
    );
    let cases: Vec<(Value, i32)> = vec![
        (
            json!([{"jsonrpc": "2.0", "id": 1, "method": "info"}]),
            INVALID_REQUEST,
        ),
        (json!({"jsonrpc": "2.0", "method": "info"}), INVALID_REQUEST),
        (
            json!({"jsonrpc": "1.0", "id": 1, "method": "info"}),
            INVALID_REQUEST,
        ),
        (
            json!({"jsonrpc": "2.0", "id": 1, "method": "get_block_template"}),
            METHOD_NOT_FOUND,
        ),
        (
            json!({"jsonrpc": "2.0", "id": 1, "method": "info", "params": [1]}),
            INVALID_PARAMS,
        ),
    ];
    let rusak = parse_perintah(b"{").unwrap_err();
    assert_eq!(code(&rusak), Some(i64::from(PARSE_ERROR)), "{rusak}");
    for (body, expected) in cases {
        let answer = parse_perintah(body.to_string().as_bytes()).unwrap_err();
        assert_eq!(code(&answer), Some(i64::from(expected)), "{answer}");
    }
    assert_eq!(METODE_PENGENDALI.len(), 12);
}

#[test]
fn requires_token_and_post() {
    let setup = Setup::start(None);
    let body = &badan("info");
    let (status, head, text) = raw(setup.addr, "POST", None, body);
    assert_eq!(status, 401);
    assert_eq!(
        serde_json::from_str::<Value>(&text).unwrap(),
        json!({"error": "unauthorized"})
    );
    assert!(head.contains("www-authenticate: bearer"), "{head}");
    assert_eq!(raw(setup.addr, "POST", Some(TOKEN_LAIN), body).0, 401);
    let (status, head, _) = raw(setup.addr, "GET", Some(TOKEN), b"");
    assert_eq!(status, 405);
    assert!(head.contains("allow: post"), "{head}");
    let (status, head, _) = raw(setup.addr, "POST", Some(TOKEN), body);
    assert_eq!(status, 200);
    assert!(head.contains("cache-control: no-store"), "{head}");
    setup.finish();
}

#[test]
fn answers_methods() {
    let setup = Setup::start(None);
    let a = setup.addr;
    let info = call(a, "info", &json!({}));
    assert_eq!(
        info["result"],
        json!({"versi": env!("CARGO_PKG_VERSION"), "versiNode": "zevrigo-node 0.1.0"})
    );
    assert_eq!(
        call(a, "keadaan_node", &json!({}))["result"]["status"],
        "mati"
    );
    let tambang = call(a, "keadaan_tambang", &json!({}));
    assert_eq!(tambang["result"]["status"], "berhenti");
    assert_eq!(tambang["result"]["blokSesiIni"], 0);

    let alamat = encode_address(Network::Regtest, OutputKind::SingleKey, &[9; 32]);
    assert_eq!(
        call(a, "periksa_alamat", &json!({"alamat": alamat}))["result"],
        json!({"sah": true, "alasan": null})
    );
    assert_eq!(
        call(a, "periksa_alamat", &json!({"alamat": ""}))["result"]["alasan"],
        "kosong"
    );
    assert_eq!(
        code(&call(a, "periksa_alamat", &json!({}))),
        Some(i64::from(INVALID_PARAMS))
    );

    assert_eq!(
        call(a, "muat_pengaturan", &json!(null))["result"]["jedaDetik"],
        120
    );
    let simpan = call(
        a,
        "simpan_pengaturan",
        &json!({"pengaturan": {"alamatHadiah": alamat, "jedaDetik": 60, "tema": "gelap"}}),
    );
    assert_eq!(simpan["result"], Value::Null, "{simpan}");
    assert_eq!(
        call(a, "muat_pengaturan", &json!({}))["result"],
        json!({"alamatHadiah": alamat, "jedaDetik": 60, "tema": "gelap"})
    );
    assert_eq!(
        code(&call(a, "simpan_pengaturan", &json!({"pengaturan": 5}))),
        Some(i64::from(INVALID_PARAMS))
    );

    let ditolak = call(a, "mulai_tambang", &json!({"alamat": alamat, "jeda": 120}));
    assert_eq!(code(&ditolak), Some(i64::from(GALAT_PERINTAH)), "{ditolak}");
    assert!(
        ditolak["error"]["message"]
            .as_str()
            .unwrap()
            .contains("belum sinkron")
    );
    assert_eq!(
        code(&call(a, "mulai_tambang", &json!({"alamat": alamat}))),
        Some(i64::from(INVALID_PARAMS))
    );
    assert_eq!(
        call(a, "berhenti_tambang", &json!({}))["result"],
        Value::Null
    );
    assert_eq!(call(a, "hentikan_node", &json!({}))["result"], Value::Null);

    let log = call(a, "log_terbaru", &json!({"sejak": null}));
    let teks: Vec<String> = log["result"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["teks"].as_str().unwrap().to_owned())
        .collect();
    assert!(
        teks.iter()
            .any(|t| t.starts_with("perintah jarak jauh: simpan_pengaturan (alamat hadiah rzvrg1")),
        "{teks:?}"
    );
    assert!(
        teks.iter()
            .any(|t| t.contains("mulai_tambang") && t.contains("jeda 120")),
        "{teks:?}"
    );
    let terakhir = log["result"].as_array().unwrap().last().unwrap()["waktu"]
        .as_u64()
        .unwrap();
    assert_eq!(
        call(a, "log_terbaru", &json!({"sejak": terakhir}))["result"],
        json!([])
    );
    assert_eq!(
        code(&call(a, "log_terbaru", &json!({"sejak": "kemarin"}))),
        Some(i64::from(INVALID_PARAMS))
    );
    setup.finish();
}

#[test]
fn token_can_be_replaced_or_removed() {
    let setup = Setup::start(None);
    let body = &badan("keadaan_node");
    assert_eq!(raw(setup.addr, "POST", Some(TOKEN), body).0, 200);
    std::fs::write(setup.dir.join("token.sha256"), hex(TOKEN_LAIN)).unwrap();
    assert_eq!(raw(setup.addr, "POST", Some(TOKEN), body).0, 401);
    assert_eq!(raw(setup.addr, "POST", Some(TOKEN_LAIN), body).0, 200);
    std::fs::remove_file(setup.dir.join("token.sha256")).unwrap();
    assert_eq!(raw(setup.addr, "POST", Some(TOKEN_LAIN), body).0, 401);
    setup.finish();
}

#[test]
fn limits_wrong_tokens_and_large_bodies() {
    let setup = Setup::start(Limits::from_rate(1000));
    let body = &badan("info");
    let status: Vec<u16> = (0..14)
        .map(|_| raw(setup.addr, "POST", Some(TOKEN_LAIN), body).0)
        .collect();
    assert!(status.iter().take(10).all(|s| *s == 401), "{status:?}");
    assert_eq!(status.last(), Some(&429), "{status:?}");
    // Token benar tidak terkena ember token salah.
    assert_eq!(raw(setup.addr, "POST", Some(TOKEN), body).0, 200);
    let besar = vec![b' '; super::MAX_BODY_BYTES + 1];
    assert_eq!(raw(setup.addr, "POST", Some(TOKEN), &besar).0, 413);
    setup.finish();

    let ketat = Setup::start(Limits::from_rate(1));
    let status: Vec<u16> = (0..6)
        .map(|_| raw(ketat.addr, "POST", Some(TOKEN), body).0)
        .collect();
    assert_eq!(status.last(), Some(&429), "{status:?}");
    ketat.finish();
}

#[test]
fn mining_marker_is_kept_until_stopped_by_command() {
    let dir = folder();
    assert_eq!(baca_lanjut(&dir), None);
    let alamat = encode_address(Network::Regtest, OutputKind::SingleKey, &[3; 32]);
    simpan_lanjut(&dir, &format!(" {alamat} "), 120).unwrap();
    assert_eq!(baca_lanjut(&dir), Some((alamat.clone(), 120)));
    std::fs::write(dir.join(BERKAS_LANJUT), "rusak").unwrap();
    assert_eq!(baca_lanjut(&dir), None);
    hapus_lanjut(&dir);
    hapus_lanjut(&dir);
    assert!(!dir.join(BERKAS_LANJUT).exists());
    let _ = std::fs::remove_dir_all(&dir);

    let setup = Setup::start(None);
    let data = setup.dir.join("data");
    // Mulai tambang yang ditolak (node belum sinkron) tidak menulis penanda.
    assert_eq!(
        code(&call(
            setup.addr,
            "mulai_tambang",
            &json!({"alamat": alamat, "jeda": 60})
        )),
        Some(i64::from(GALAT_PERINTAH))
    );
    assert_eq!(baca_lanjut(&data), None);
    for metode in ["berhenti_tambang", "hentikan_node"] {
        simpan_lanjut(&data, &alamat, 60).unwrap();
        assert_eq!(call(setup.addr, metode, &json!({}))["result"], Value::Null);
        assert_eq!(baca_lanjut(&data), None, "{metode}");
    }
    setup.finish();
}

/// Node tiruan untuk `saldo`: tinggi 200, tiga output ke payload `0x33..` (dua matang, satu belum).
struct NodeSaldo;

fn h(byte: u8) -> Hex32 {
    Hex32::new([byte; 32])
}

fn utxo(index: u32, zvrg: u128, spendable_at: u64) -> UtxoInfo {
    UtxoInfo {
        txid: h(0x11),
        index,
        amount: Quanta::new(Amount::from_quanta(zvrg * QUANTA_PER_ZVRG)),
        kind: KindNumber::new(OutputKind::SingleKey),
        payload: h(0x33),
        height: spendable_at.saturating_sub(100),
        is_coinbase: true,
        confirmations: 1,
        spendable_at,
    }
}

impl RpcBackend for NodeSaldo {
    fn chain_info(&self) -> Result<ChainInfo, RpcError> {
        Ok(ChainInfo {
            network: NetworkName::new(Network::Regtest),
            height: 200,
            best_block_id: h(0xb7),
            chainwork: h(0x00),
            next_target: CompactHex::new(0x207f_ffff),
            median_time_past: 1_793_145_600,
            mined_total: Quanta::new(Amount::from_quanta(5)),
            total_supply: Quanta::new(Amount::from_quanta(6)),
            mempool_count: 0,
        })
    }

    fn block_hash(&self, _height: u64) -> Result<Hex32, RpcError> {
        Err(RpcError::NotFound)
    }

    fn block(&self, _block_id: Hex32) -> Result<BlockInfo, RpcError> {
        Err(RpcError::NotFound)
    }

    fn block_template(&self, _params: &GetBlockTemplateParams) -> Result<BlockTemplate, RpcError> {
        Err(RpcError::NotFound)
    }

    fn submit_block(&self, _block: &[u8]) -> Result<SubmitBlockResult, RpcError> {
        Err(RpcError::NotFound)
    }

    fn send_raw_transaction(&self, _tx: &[u8]) -> Result<SendRawTransactionResult, RpcError> {
        Err(RpcError::NotFound)
    }

    fn mempool(&self) -> Result<MempoolInfo, RpcError> {
        Ok(MempoolInfo { txids: Vec::new() })
    }

    fn scan_utxos(&self, payloads: &[Hex32]) -> Result<Vec<UtxoInfo>, RpcError> {
        if payloads == [h(0x33)] {
            Ok(vec![utxo(0, 3, 150), utxo(1, 2, 201), utxo(2, 7, 260)])
        } else {
            Ok(Vec::new())
        }
    }

    fn scan_mempool(&self, _payloads: &[Hex32]) -> Result<MempoolScan, RpcError> {
        Ok(MempoolScan {
            spent: Vec::new(),
            outputs: Vec::new(),
        })
    }
}

#[test]
fn reports_balance_of_reward_address() {
    let rt = Runtime::new().unwrap();
    let node = rt
        .block_on(RpcServer::start(
            SocketAddr::from(([127, 0, 0, 1], 0)),
            Arc::new(NodeSaldo),
        ))
        .unwrap();
    let setup = Setup::start_node(None, format!("http://{}", node.local_addr()));
    let ada = encode_address(Network::Regtest, OutputKind::SingleKey, &[0x33; 32]);
    let jawab = call(setup.addr, "saldo", &json!({"alamat": ada}));
    assert_eq!(
        jawab["result"],
        json!({"tinggi": 200, "matang": "5", "belumMatang": "7", "jumlahOutput": 3}),
        "{jawab}"
    );
    let kosong = encode_address(Network::Regtest, OutputKind::SingleKey, &[0x44; 32]);
    assert_eq!(
        call(setup.addr, "saldo", &json!({"alamat": kosong}))["result"],
        json!({"tinggi": 200, "matang": "0", "belumMatang": "0", "jumlahOutput": 0})
    );
    let testnet = encode_address(Network::Testnet, OutputKind::SingleKey, &[0x33; 32]);
    assert_eq!(
        code(&call(setup.addr, "saldo", &json!({"alamat": testnet}))),
        Some(i64::from(INVALID_PARAMS))
    );
    assert_eq!(
        code(&call(setup.addr, "saldo", &json!({}))),
        Some(i64::from(INVALID_PARAMS))
    );
    setup.finish();
    rt.block_on(node.stop());

    let mati = Setup::start(None);
    let ada = encode_address(Network::Regtest, OutputKind::SingleKey, &[0x33; 32]);
    assert_eq!(
        code(&call(mati.addr, "saldo", &json!({"alamat": ada}))),
        Some(i64::from(GALAT_PERINTAH))
    );
    mati.finish();
}
