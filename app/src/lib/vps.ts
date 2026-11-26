// Mode VPS aplikasi Zevrigo Node (Android, RENCANA_T17 bagian 13, DEC-46; Q199b): node dan penambang berjalan di VPS pengguna,
// aplikasi mengendalikannya lewat perintah Tauri `vps_*` (src-tauri/src/perintah_vps.rs, fitur Cargo `vps`). Kata sandi
// aplikasi, password VPS, kunci SSH, dan token pengendali tidak pernah sampai ke antarmuka. Di browser (`npm run dev` dengan
// `?mode=vps`) dan di test memakai tiruan `createMockVpsApi`.

import { invoke } from '@tauri-apps/api/core';
import {
  diTauri,
  type Backend,
  type BarisLog,
  type HasilPeriksaAlamat,
  type InfoAplikasi,
  type KeadaanNode,
  type KeadaanTambang,
  type Pengaturan,
  type Tema,
} from './backend';
import { createMockBackend } from './mockBackend';

export type TahapVps = 'baru' | 'sambung' | 'pasang' | 'amankan' | 'terkunci' | 'siap';

export interface PemeriksaanVps {
  arsitektur: string;
  os: string;
  versiOs: string;
  namaOs: string;
  systemd: boolean;
  root: boolean;
  sudo: boolean;
  unduh: boolean;
  ca: boolean;
  terpasang: boolean;
  diskBebasMb: number;
  ramMb: number;
}

export interface InfoVps {
  tahap: TahapVps;
  host: string | null;
  port: number | null;
  pengguna: string | null;
  sidik: string | null;
  pemeriksaan: PemeriksaanVps | null;
  masalah: string[];
  amankanSelesai: boolean;
  paketSha256: string;
  sandiMin: number;
  sandiMaks: number;
  versiAplikasi: string;
}

export interface StatusVps {
  versi: string;
  layanan: string;
  aktifSejak: string;
  cpu: number;
  muatan1m: string;
  ramTotalMb: number;
  ramTersediaMb: number;
  diskTotalMb: number;
  diskBebasMb: number;
  uptimeDetik: number;
  nodePublik: string;
  kuotaCpu: string;
  loginPassword: string;
}

export interface SaldoVps {
  tinggi: number;
  matang: string;
  belumMatang: string;
  jumlahOutput: number;
}

export type PerintahVps =
  | { jenis: 'perbarui' }
  | { jenis: 'batas_cpu'; persen: number }
  | { jenis: 'node_publik_nyala'; ip: string; port: number }
  | { jenis: 'node_publik_mati' }
  | { jenis: 'restart_vps' };

export interface GalatVps {
  kode: string;
  pesan: string;
}

export interface VpsApi {
  keadaan(): Promise<InfoVps>;
  buatSandi(sandi: string): Promise<InfoVps>;
  batal(): Promise<InfoVps>;
  sambung(host: string, port: number, pengguna: string, sandi: string): Promise<InfoVps>;
  pasang(): Promise<InfoVps>;
  amankan(): Promise<InfoVps>;
  lewatiAmankan(): Promise<InfoVps>;
  buka(sandi: string): Promise<InfoVps>;
  kunci(): Promise<InfoVps>;
  rpc<T>(metode: string, params: Record<string, unknown>): Promise<T>;
  status(): Promise<StatusVps>;
  kendali(perintah: PerintahVps): Promise<string>;
  gantiToken(): Promise<void>;
  ekspor(): Promise<string>;
  impor(teks: string, sandi: string): Promise<InfoVps>;
  lupakan(): Promise<InfoVps>;
  cabutAkses(): Promise<InfoVps>;
  copot(hapus: boolean): Promise<InfoVps>;
  bukaTautan(url: string): Promise<void>;
}

/** Perintah Tauri di src-tauri/src/perintah_vps.rs. Argumen satu kata agar nama JavaScript sama dengan nama Rust. */
export function createTauriVpsApi(): VpsApi {
  return {
    keadaan: () => invoke<InfoVps>('vps_keadaan'),
    buatSandi: (sandi: string) => invoke<InfoVps>('vps_buat_sandi', { sandi }),
    batal: () => invoke<InfoVps>('vps_batal'),
    sambung: (host: string, port: number, pengguna: string, sandi: string) =>
      invoke<InfoVps>('vps_sambung', { host, port, pengguna, sandi }),
    pasang: () => invoke<InfoVps>('vps_pasang'),
    amankan: () => invoke<InfoVps>('vps_amankan'),
    lewatiAmankan: () => invoke<InfoVps>('vps_lewati_amankan'),
    buka: (sandi: string) => invoke<InfoVps>('vps_buka', { sandi }),
    kunci: () => invoke<InfoVps>('vps_kunci'),
    rpc: <T>(metode: string, params: Record<string, unknown>) =>
      invoke<T>('vps_rpc', { metode, params }),
    status: () => invoke<StatusVps>('vps_status'),
    kendali: (perintah: PerintahVps) => invoke<string>('vps_kendali', { perintah }),
    gantiToken: async () => {
      await invoke('vps_ganti_token');
    },
    ekspor: () => invoke<string>('vps_ekspor'),
    impor: (teks: string, sandi: string) => invoke<InfoVps>('vps_impor', { teks, sandi }),
    lupakan: () => invoke<InfoVps>('vps_lupakan'),
    cabutAkses: () => invoke<InfoVps>('vps_cabut_akses'),
    copot: (hapus: boolean) => invoke<InfoVps>('vps_copot', { hapus }),
    bukaTautan: async (url: string) => {
      await invoke('buka_tautan', { url });
    },
  };
}

/** Galat dari perintah Tauri (`{kode, pesan}`), tiruan, atau JavaScript. */
export function keGalat(e: unknown): GalatVps {
  if (typeof e === 'object' && e !== null && 'kode' in e && 'pesan' in e) {
    const o = e as { kode: unknown; pesan: unknown };
    return { kode: String(o.kode), pesan: String(o.pesan) };
  }
  if (e instanceof Error) {
    return { kode: 'sistem', pesan: e.message };
  }
  return { kode: 'sistem', pesan: String(e) };
}

const KUNCI_TEMA = 'zevrigo-node-tema';

/** Tema tampilan mode VPS disimpan di HP (bukan di VPS). */
export function temaLokal(): Tema {
  if (typeof localStorage === 'undefined') {
    return 'sistem';
  }
  const tema = localStorage.getItem(KUNCI_TEMA);
  return tema === 'terang' || tema === 'gelap' ? tema : 'sistem';
}

export function simpanTemaLokal(tema: Tema): void {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(KUNCI_TEMA, tema);
  }
}

function jedaSah(jeda: unknown): 60 | 120 | 300 {
  return jeda === 60 || jeda === 300 ? jeda : 120;
}

/** Backend kartu Node, Menambang, dan Log (kontrak `Backend` desktop) lewat pengendali di VPS. */
export function createVpsBackend(api: VpsApi): Backend {
  return {
    async info(): Promise<InfoAplikasi> {
      const [lokal, jauh] = await Promise.all([
        api.keadaan(),
        api.rpc<{ versi: string; versiNode: string }>('info', {}),
      ]);
      return { versi: lokal.versiAplikasi, versiNode: jauh.versiNode };
    },
    async muatPengaturan(): Promise<Pengaturan> {
      const p = await api.rpc<Partial<Pengaturan>>('muat_pengaturan', {});
      return {
        alamatHadiah: typeof p.alamatHadiah === 'string' ? p.alamatHadiah : '',
        jedaDetik: jedaSah(p.jedaDetik),
        tema: temaLokal(),
      };
    },
    async simpanPengaturan(p: Pengaturan): Promise<void> {
      simpanTemaLokal(p.tema);
      await api.rpc('simpan_pengaturan', {
        pengaturan: { alamatHadiah: p.alamatHadiah, jedaDetik: p.jedaDetik, tema: 'sistem' },
      });
    },
    async nyalakanNode(): Promise<void> {
      await api.rpc('nyalakan_node', {});
    },
    async hentikanNode(): Promise<void> {
      await api.rpc('hentikan_node', {});
    },
    keadaanNode: () => api.rpc<KeadaanNode>('keadaan_node', {}),
    periksaAlamat: (alamat: string) =>
      api.rpc<HasilPeriksaAlamat>('periksa_alamat', { alamat }),
    async mulaiTambang(alamat: string, jedaDetik: number): Promise<void> {
      await api.rpc('mulai_tambang', { alamat, jeda: jedaDetik });
    },
    async berhentiTambang(): Promise<void> {
      await api.rpc('berhenti_tambang', {});
    },
    keadaanTambang: () => api.rpc<KeadaanTambang>('keadaan_tambang', {}),
    logTerbaru: (sejakWaktu: number | null) =>
      api.rpc<BarisLog[]>('log_terbaru', { sejak: sejakWaktu }),
    // Folder data ada di VPS (/var/lib/demo-zevrigo); tombolnya tidak tampil di mode VPS.
    async bukaFolderData(): Promise<void> {},
    bukaTautan: (url: string) => api.bukaTautan(url),
    async salinTeks(teks: string): Promise<void> {
      await navigator.clipboard.writeText(teks);
    },
    // Menutup aplikasi tidak menghentikan node di VPS.
    saatAkanTutup(): () => void {
      return () => {};
    },
    async selesaiTutup(): Promise<void> {},
  };
}

/** ZVRG desimal dari pengendali ("13.777828687378155611") untuk tampilan: ribuan titik, desimal koma, 8 angka. */
export function formatZvrg(nilai: string): string {
  const [bulat = '0', pecahan = ''] = nilai.trim().split('.');
  const ribuan = bulat.replace(/\B(?=(\d{3})+(?!\d))/g, '.');
  const sisa = pecahan.slice(0, 8).replace(/0+$/, '');
  return sisa ? `${ribuan},${sisa}` : ribuan;
}

export function formatMb(mb: number): string {
  if (!Number.isFinite(mb) || mb < 0) {
    return '-';
  }
  if (mb >= 1024) {
    const persepuluh = Math.round((mb * 10) / 1024);
    return `${Math.floor(persepuluh / 10)},${persepuluh % 10} GB`;
  }
  return `${Math.round(mb)} MB`;
}

export function formatDurasi(detik: number): string {
  if (!Number.isFinite(detik) || detik < 0) {
    return '-';
  }
  const hari = Math.floor(detik / 86400);
  const jam = Math.floor((detik % 86400) / 3600);
  const menit = Math.floor((detik % 3600) / 60);
  if (hari > 0) {
    return `${hari} hari ${jam} jam`;
  }
  if (jam > 0) {
    return `${jam} jam ${menit} menit`;
  }
  return `${menit} menit`;
}

/** Batas CPU dari systemd (`500ms` per detik = 50%); `null` bila tanpa batas. */
export function persenKuota(kuota: string): number | null {
  const cocok = /^(\d+)ms$/.exec(kuota.trim());
  return cocok ? Math.round(Number(cocok[1]) / 10) : null;
}

export const PAKET_TIRUAN = '7d90a27fc6e7cd67eda944291a77fdb9cf1fc13ba7b0765a3e1a4cffb333306b';

function cekTiruan(sandi: string): string {
  let h = 5381;
  for (const c of sandi) {
    h = (Math.imul(h, 33) ^ c.charCodeAt(0)) >>> 0;
  }
  return h.toString(16);
}

/** Tiruan untuk browser dan test: tahap, brankas, dan pengendali (node tiruan `createMockBackend`). */
export function createMockVpsApi(): VpsApi {
  const node = createMockBackend();
  const SIDIK = 'SHA256:TiruanTiruanTiruanTiruanTiruanTiruanTiruan';
  const tunda = (ms: number) => new Promise<void>((selesai) => setTimeout(selesai, ms));
  const galat = (kode: string, pesan: string): GalatVps => ({ kode, pesan });
  let brankas: { teks: string; sandi: string } | null = null;
  let terbuka = false;
  let lewati = false;
  let amankanSelesai = false;
  let terpasangVps = false;
  let kuota = '500ms';
  let nodePublik = '';
  let sasaran = { host: '', port: 22, pengguna: '' };
  let awal: {
    sandi: string;
    host: string | null;
    port: number | null;
    pengguna: string | null;
    pemeriksaan: PemeriksaanVps | null;
  } | null = null;

  function info(): InfoVps {
    const dasar: Omit<InfoVps, 'tahap'> = {
      host: null,
      port: null,
      pengguna: null,
      sidik: null,
      pemeriksaan: null,
      masalah: [],
      amankanSelesai: false,
      paketSha256: PAKET_TIRUAN,
      sandiMin: 8,
      sandiMaks: 256,
      versiAplikasi: '0.1.0',
    };
    if (terbuka) {
      return {
        ...dasar,
        tahap: amankanSelesai || lewati ? 'siap' : 'amankan',
        host: sasaran.host,
        port: sasaran.port,
        pengguna: sasaran.pengguna,
        sidik: SIDIK,
        amankanSelesai,
      };
    }
    if (awal !== null) {
      return {
        ...dasar,
        tahap: awal.pemeriksaan ? 'pasang' : 'sambung',
        host: awal.host,
        port: awal.port,
        pengguna: awal.pengguna,
        sidik: awal.pemeriksaan ? SIDIK : null,
        pemeriksaan: awal.pemeriksaan,
      };
    }
    return { ...dasar, tahap: brankas ? 'terkunci' : 'baru' };
  }

  function tulisBrankas(sandi: string): void {
    const teks = JSON.stringify({ jenis: 'zevrigo-node-vps-tiruan', ...sasaran, amankanSelesai, cek: cekTiruan(sandi) });
    brankas = { teks, sandi };
  }

  function wajibTerbuka(): void {
    if (!terbuka) {
      throw galat('terkunci', 'aplikasi terkunci; masukkan kata sandi aplikasi');
    }
  }

  async function rpcTiruan(metode: string, params: Record<string, unknown>): Promise<unknown> {
    wajibTerbuka();
    switch (metode) {
      case 'info':
        return { versi: '0.1.0', versiNode: 'zevrigo-node 0.1.0 (tiruan)' };
      case 'keadaan_node':
        return { ...(await node.keadaanNode()), folderData: '/var/lib/demo-zevrigo' };
      case 'keadaan_tambang':
        return node.keadaanTambang();
      case 'log_terbaru':
        return node.logTerbaru(typeof params.sejak === 'number' ? params.sejak : null);
      case 'muat_pengaturan':
        return node.muatPengaturan();
      case 'simpan_pengaturan':
        return node.simpanPengaturan(params.pengaturan as Pengaturan);
      case 'periksa_alamat':
        return node.periksaAlamat(String(params.alamat ?? ''));
      case 'nyalakan_node':
        return node.nyalakanNode();
      case 'hentikan_node':
        return node.hentikanNode();
      case 'mulai_tambang':
        return node.mulaiTambang(String(params.alamat ?? ''), Number(params.jeda));
      case 'berhenti_tambang':
        return node.berhentiTambang();
      case 'saldo': {
        const t = await node.keadaanTambang();
        const n = await node.keadaanNode();
        return {
          tinggi: n.tinggiLokal ?? 0,
          matang: '0',
          belumMatang: `${t.blokSesiIni * 12}.5`,
          jumlahOutput: t.blokSesiIni,
        } satisfies SaldoVps;
      }
      default:
        throw galat('pengendali', `metode pengendali tidak dikenal: ${metode}`);
    }
  }

  return {
    async keadaan() {
      return info();
    },
    async buatSandi(sandi: string) {
      if (brankas !== null || terbuka) {
        throw galat('tahap', 'langkah tidak sesuai: data VPS sudah ada di HP ini');
      }
      if ([...sandi].length < 8) {
        throw galat('masukan', 'kata sandi aplikasi 8 sampai 256 karakter');
      }
      awal = { sandi, host: null, port: null, pengguna: null, pemeriksaan: null };
      return info();
    },
    async batal() {
      awal = null;
      return info();
    },
    async sambung(host: string, port: number, pengguna: string, sandi: string) {
      if (awal === null) {
        throw galat('tahap', 'langkah tidak sesuai: buat kata sandi aplikasi dulu');
      }
      if (host.trim() === '' || pengguna.trim() === '') {
        throw galat('masukan', 'alamat VPS tidak sah');
      }
      await tunda(600);
      if (sandi === 'salah') {
        throw galat('masuk_ditolak', 'VPS menolak nama pengguna, password, atau kunci');
      }
      awal = {
        ...awal,
        host: host.trim(),
        port,
        pengguna: pengguna.trim(),
        pemeriksaan: {
          arsitektur: 'x86_64',
          os: 'ubuntu',
          versiOs: '24.04',
          namaOs: 'Ubuntu 24.04 LTS (tiruan)',
          systemd: true,
          root: pengguna.trim() === 'root',
          sudo: true,
          unduh: true,
          ca: true,
          terpasang: terpasangVps,
          diskBebasMb: 20480,
          ramMb: 1980,
        },
      };
      return info();
    },
    async pasang() {
      if (awal === null || awal.pemeriksaan === null) {
        throw galat('tahap', 'langkah tidak sesuai: sambung ke VPS dulu');
      }
      await tunda(2500);
      sasaran = { host: awal.host ?? '', port: awal.port ?? 22, pengguna: awal.pengguna ?? '' };
      amankanSelesai = false;
      lewati = false;
      tulisBrankas(awal.sandi);
      awal = null;
      terbuka = true;
      terpasangVps = true;
      await node.nyalakanNode();
      return info();
    },
    async amankan() {
      wajibTerbuka();
      await tunda(500);
      amankanSelesai = true;
      if (brankas !== null) {
        tulisBrankas(brankas.sandi);
      }
      return info();
    },
    async lewatiAmankan() {
      wajibTerbuka();
      lewati = true;
      return info();
    },
    async buka(sandi: string) {
      if (brankas === null) {
        throw galat('brankas', 'data VPS di HP: tidak terbaca');
      }
      await tunda(300);
      if (sandi !== brankas.sandi) {
        throw galat('sandi_salah', 'kata sandi aplikasi salah');
      }
      terbuka = true;
      lewati = false;
      return info();
    },
    async kunci() {
      terbuka = false;
      awal = null;
      return info();
    },
    async rpc<T>(metode: string, params: Record<string, unknown>): Promise<T> {
      return (await rpcTiruan(metode, params)) as T;
    },
    async status() {
      wajibTerbuka();
      return {
        versi: 'demo-zevrigo-vps zevrigo-demo faa9c3a (tiruan)',
        layanan: 'active',
        aktifSejak: '',
        cpu: 1,
        muatan1m: '0.12',
        ramTotalMb: 1980,
        ramTersediaMb: 1400,
        diskTotalMb: 25600,
        diskBebasMb: 20480,
        uptimeDetik: 3 * 86400 + 5 * 3600,
        nodePublik,
        kuotaCpu: kuota,
        loginPassword: amankanSelesai ? 'no' : 'yes',
      };
    },
    async kendali(perintah: PerintahVps) {
      wajibTerbuka();
      await tunda(perintah.jenis === 'perbarui' ? 2000 : 400);
      switch (perintah.jenis) {
        case 'perbarui':
          return 'terpasang: rilis-tiruan (demo-zevrigo-vps tiruan), sebelumnya rilis-lama';
        case 'batas_cpu':
          if (perintah.persen < 10 || perintah.persen > 100) {
            throw galat('masukan', 'batas CPU harus 10 sampai 100');
          }
          kuota = `${perintah.persen * 10}ms`;
          return `batas CPU: ${perintah.persen}%`;
        case 'node_publik_nyala':
          nodePublik = `/ip4/${perintah.ip}/tcp/${perintah.port}`;
          return `node publik: ${nodePublik}`;
        case 'node_publik_mati':
          nodePublik = '';
          return 'node publik: mati';
        case 'restart_vps':
          return 'VPS restart dalam 5 detik';
      }
      throw galat('masukan', 'perintah tidak dikenal');
    },
    async gantiToken() {
      wajibTerbuka();
      await tunda(300);
    },
    async ekspor() {
      wajibTerbuka();
      if (brankas === null) {
        throw galat('brankas', 'data VPS di HP: tidak terbaca');
      }
      return brankas.teks;
    },
    async impor(teks: string, sandi: string) {
      if (brankas !== null || terbuka) {
        throw galat('tahap', 'langkah tidak sesuai: data VPS sudah ada di HP ini');
      }
      type DataTiruan = {
        jenis?: unknown;
        host?: unknown;
        port?: unknown;
        pengguna?: unknown;
        amankanSelesai?: unknown;
        cek?: unknown;
      };
      let data: DataTiruan | null = null;
      try {
        const nilai: unknown = JSON.parse(teks);
        data = typeof nilai === 'object' && nilai !== null ? (nilai as DataTiruan) : null;
      } catch {
        data = null;
      }
      if (data === null || data.jenis !== 'zevrigo-node-vps-tiruan') {
        throw galat('brankas', 'data VPS di HP: bukan data VPS Zevrigo Node');
      }
      if (data.cek !== cekTiruan(sandi)) {
        throw galat('sandi_salah', 'kata sandi aplikasi salah');
      }
      sasaran = { host: String(data.host), port: Number(data.port), pengguna: String(data.pengguna) };
      amankanSelesai = data.amankanSelesai === true;
      brankas = { teks, sandi };
      terbuka = true;
      lewati = false;
      return info();
    },
    async lupakan() {
      brankas = null;
      terbuka = false;
      awal = null;
      return info();
    },
    async cabutAkses() {
      wajibTerbuka();
      await tunda(400);
      brankas = null;
      terbuka = false;
      return info();
    },
    async copot() {
      wajibTerbuka();
      await tunda(800);
      terpasangVps = false;
      await node.hentikanNode();
      brankas = null;
      terbuka = false;
      return info();
    },
    bukaTautan: (url: string) => node.bukaTautan(url),
  };
}

let apiAktif: VpsApi | null = null;

/** API VPS: perintah Tauri di aplikasi, tiruan di browser. */
export function ambilVpsApi(): VpsApi {
  if (apiAktif === null) {
    apiAktif = diTauri() ? createTauriVpsApi() : createMockVpsApi();
  }
  return apiAktif;
}

/** Untuk test. */
export function aturVpsApi(api: VpsApi): void {
  apiAktif = api;
}
