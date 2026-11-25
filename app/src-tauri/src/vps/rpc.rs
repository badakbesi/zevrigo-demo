//! JSON-RPC pengendali (DEC-45, Q196a) lewat terowongan SSH ke 127.0.0.1:28808: HTTP/1.1 sederhana di atas kanal SSH, tanpa
//! pustaka HTTP.

use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

use super::sesi::{Galat, Sesi, sambung};

/// Port pengendali di VPS (hanya 127.0.0.1).
const PORT_PENGENDALI: u32 = 28808;
/// Batas ukuran jawaban pengendali.
const BATAS_JAWABAN: usize = 4 << 20;
/// Batas waktu satu panggilan.
const BATAS_RPC: Duration = Duration::from_secs(60);

/// Metode pengendali yang boleh dipanggil aplikasi (tombol aplikasi desktop ditambah `saldo`).
pub const METODE: [&str; 12] = [
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
    "saldo",
];

/// Token pengendali yang dikirim aplikasi: 32 sampai 256 huruf/angka.
pub(super) fn token_sah(token: &str) -> bool {
    (32..=256).contains(&token.len()) && token.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Permintaan HTTP satu kali pakai ke pengendali.
pub(super) fn permintaan_http(token: &str, badan: &[u8]) -> Vec<u8> {
    let mut permintaan = format!(
        "POST / HTTP/1.1\r\nHost: 127.0.0.1:{PORT_PENGENDALI}\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        badan.len()
    )
    .into_bytes();
    permintaan.extend_from_slice(badan);
    permintaan
}

/// Status HTTP dan badan jawaban (Content-Length, chunked, atau sampai sambungan ditutup).
pub(super) fn urai_http(jawaban: &[u8]) -> Result<(u16, Vec<u8>), Galat> {
    let rusak = || Galat::Pengendali("jawaban HTTP tidak terbaca".to_owned());
    let batas = jawaban
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(rusak)?;
    let kepala =
        std::str::from_utf8(jawaban.get(..batas).ok_or_else(rusak)?).map_err(|_| rusak())?;
    let badan = jawaban.get(batas.saturating_add(4)..).ok_or_else(rusak)?;
    let mut baris = kepala.split("\r\n");
    let mut status_baris = baris.next().ok_or_else(rusak)?.split(' ');
    if !status_baris.next().is_some_and(|v| v.starts_with("HTTP/")) {
        return Err(rusak());
    }
    let status: u16 = status_baris
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(rusak)?;
    let mut panjang = None;
    let mut chunked = false;
    for b in baris {
        let Some((nama, nilai)) = b.split_once(':') else {
            continue;
        };
        let nilai = nilai.trim();
        if nama.eq_ignore_ascii_case("content-length") {
            panjang = Some(nilai.parse::<usize>().map_err(|_| rusak())?);
        } else if nama.eq_ignore_ascii_case("transfer-encoding")
            && nilai.eq_ignore_ascii_case("chunked")
        {
            chunked = true;
        }
    }
    let isi = if chunked {
        urai_chunked(badan).ok_or_else(rusak)?
    } else if let Some(n) = panjang {
        badan.get(..n).ok_or_else(rusak)?.to_vec()
    } else {
        badan.to_vec()
    };
    Ok((status, isi))
}

fn urai_chunked(mut sisa: &[u8]) -> Option<Vec<u8>> {
    let mut isi = Vec::new();
    loop {
        let akhir = sisa.windows(2).position(|w| w == b"\r\n")?;
        let ukuran_teks = std::str::from_utf8(sisa.get(..akhir)?).ok()?;
        let ukuran = usize::from_str_radix(ukuran_teks.split(';').next()?.trim(), 16).ok()?;
        sisa = sisa.get(akhir.saturating_add(2)..)?;
        if ukuran == 0 {
            return Some(isi);
        }
        isi.extend_from_slice(sisa.get(..ukuran)?);
        sisa = sisa.get(ukuran.saturating_add(2)..)?;
    }
}

/// Hasil JSON-RPC dari jawaban HTTP pengendali.
pub(super) fn hasil_rpc(jawaban: &[u8]) -> Result<Value, Galat> {
    let (status, badan) = urai_http(jawaban)?;
    if status == 401 {
        return Err(Galat::TokenDitolak);
    }
    if status != 200 {
        return Err(Galat::Pengendali(format!("HTTP {status}")));
    }
    let mut nilai: Value = serde_json::from_slice(&badan)
        .map_err(|e| Galat::Pengendali(format!("JSON tidak terbaca: {e}")))?;
    if let Some(galat) = nilai.get("error") {
        let kode = galat.get("code").and_then(Value::as_i64).unwrap_or(0);
        let pesan = galat
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("tanpa pesan");
        return Err(Galat::Pengendali(format!("{pesan} (kode {kode})")));
    }
    Ok(nilai
        .get_mut("result")
        .map(Value::take)
        .unwrap_or(Value::Null))
}

impl Sesi {
    /// Memanggil metode pengendali dengan token aplikasi lewat terowongan SSH; mengembalikan `result` JSON-RPC.
    ///
    /// # Errors
    ///
    /// [`Galat::Masukan`] (metode, token, atau params tidak sah), [`Galat::TokenDitolak`], [`Galat::Pengendali`], galat
    /// sambungan.
    pub async fn rpc(&self, token: &str, metode: &str, params: Value) -> Result<Value, Galat> {
        if !METODE.contains(&metode) {
            return Err(Galat::Masukan(format!(
                "metode pengendali tidak dikenal: {metode}"
            )));
        }
        if !token_sah(token) {
            return Err(Galat::Masukan("token pengendali tidak sah".to_owned()));
        }
        if !params.is_object() {
            return Err(Galat::Masukan("params pengendali harus objek".to_owned()));
        }
        let badan = serde_json::to_vec(
            &json!({"jsonrpc": "2.0", "id": 1, "method": metode, "params": params}),
        )
        .map_err(|e| Galat::Pengendali(e.to_string()))?;
        let permintaan = permintaan_http(token, &badan);
        let kerja = async {
            let kanal = self
                .handle
                .channel_open_direct_tcpip("127.0.0.1", PORT_PENGENDALI, "127.0.0.1", 0)
                .await
                .map_err(sambung)?;
            let mut aliran = std::pin::pin!(kanal.into_stream());
            aliran.write_all(&permintaan).await.map_err(sambung)?;
            aliran.flush().await.map_err(sambung)?;
            let mut jawaban = Vec::new();
            let mut potongan = [0_u8; 8192];
            loop {
                let n = aliran.read(&mut potongan).await.map_err(sambung)?;
                if n == 0 {
                    break;
                }
                if jawaban.len().saturating_add(n) > BATAS_JAWABAN {
                    return Err(Galat::Pengendali("jawaban terlalu besar".to_owned()));
                }
                jawaban.extend_from_slice(potongan.get(..n).unwrap_or(&[]));
            }
            Ok(jawaban)
        };
        let jawaban = tokio::time::timeout(BATAS_RPC, kerja)
            .await
            .map_err(|_| Galat::BatasWaktu("pengendali di VPS"))??;
        hasil_rpc(&jawaban)
    }
}
