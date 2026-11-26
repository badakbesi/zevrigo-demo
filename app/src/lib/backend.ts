import { invoke } from '@tauri-apps/api/core';
import { createMockBackend, type MockBackendKontrol } from './mockBackend';

export type StatusNode = 'mati' | 'menyala' | 'menyambung' | 'mengejar' | 'sinkron' | 'galat';
export type StatusTambang = 'berhenti' | 'menambang' | 'menunggu';
export type Tema = 'sistem' | 'terang' | 'gelap';

export interface KeadaanNode {
  status: StatusNode;
  tinggiLokal: number | null;
  tinggiJaringan: number | null;
  rekan: number;
  folderData: string;
  pesanGalat: string | null;
  detailGalat: string | null;
}

export interface BlokDitemukan {
  tinggi: number;
  idBlok: string;
  waktu: number;
}

export interface KeadaanTambang {
  status: StatusTambang;
  alamat: string | null;
  jedaDetik: number;
  sisaJedaDetik: number | null;
  blokSesiIni: number;
  blokTerakhir: BlokDitemukan[];
  pesanGalat: string | null;
  detailGalat: string | null;
}

export interface BarisLog {
  waktu: number;
  sumber: 'node' | 'penambang' | 'aplikasi';
  tingkat: 'info' | 'peringatan' | 'galat';
  teks: string;
}

export interface Pengaturan {
  alamatHadiah: string;
  jedaDetik: 60 | 120 | 300;
  tema: Tema;
}

export interface HasilPeriksaAlamat {
  sah: boolean;
  alasan: 'kosong' | 'format' | 'jaringan-lain' | null;
}

export interface InfoAplikasi {
  versi: string;
  versiNode: string;
}

export interface Backend {
  info(): Promise<InfoAplikasi>;
  muatPengaturan(): Promise<Pengaturan>;
  simpanPengaturan(p: Pengaturan): Promise<void>;
  nyalakanNode(): Promise<void>;
  hentikanNode(): Promise<void>;
  keadaanNode(): Promise<KeadaanNode>;
  periksaAlamat(alamat: string): Promise<HasilPeriksaAlamat>;
  mulaiTambang(alamat: string, jedaDetik: number): Promise<void>;
  berhentiTambang(): Promise<void>;
  keadaanTambang(): Promise<KeadaanTambang>;
  logTerbaru(sejakWaktu: number | null): Promise<BarisLog[]>;
  bukaFolderData(): Promise<void>;
  bukaTautan(url: string): Promise<void>;
  salinTeks(teks: string): Promise<void>;
  saatAkanTutup(cb: () => void): () => void;
  selesaiTutup(): Promise<void>;
}

/** Nama perintah Tauri di src-tauri/src/perintah.rs. Argumen satu kata agar nama JavaScript sama dengan nama Rust. */
function createTauriBackend(): Backend {
  return {
    info: () => invoke<InfoAplikasi>('info'),
    muatPengaturan: () => invoke<Pengaturan>('muat_pengaturan'),
    simpanPengaturan: async (p: Pengaturan) => {
      await invoke('simpan_pengaturan', { pengaturan: p });
    },
    nyalakanNode: async () => {
      await invoke('nyalakan_node');
    },
    hentikanNode: async () => {
      await invoke('hentikan_node');
    },
    keadaanNode: () => invoke<KeadaanNode>('keadaan_node'),
    periksaAlamat: (alamat: string) => invoke<HasilPeriksaAlamat>('periksa_alamat', { alamat }),
    mulaiTambang: async (alamat: string, jedaDetik: number) => {
      await invoke('mulai_tambang', { alamat, jeda: jedaDetik });
    },
    berhentiTambang: async () => {
      await invoke('berhenti_tambang');
    },
    keadaanTambang: () => invoke<KeadaanTambang>('keadaan_tambang'),
    logTerbaru: (sejakWaktu: number | null) => invoke<BarisLog[]>('log_terbaru', { sejak: sejakWaktu }),
    bukaFolderData: async () => {
      await invoke('buka_folder_data');
    },
    bukaTautan: async (url: string) => {
      await invoke('buka_tautan', { url });
    },
    salinTeks: async (teks: string) => {
      await navigator.clipboard.writeText(teks);
    },
    // Tanpa izin event (tanpa capabilities): backend menandai jendela akan ditutup, antarmuka memeriksanya setiap 500 ms.
    saatAkanTutup(cb: () => void): () => void {
      let dipanggil = false;
      const id = setInterval(() => {
        void invoke<boolean>('akan_tutup').then((akan) => {
          if (akan && !dipanggil) {
            dipanggil = true;
            cb();
          }
        });
      }, 500);
      return () => {
        clearInterval(id);
      };
    },
    selesaiTutup: async () => {
      await invoke('selesai_tutup');
    },
  };
}

/** `true` bila berjalan di jendela Tauri (bukan browser `npm run dev` atau test). */
export function diTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/** `desktop`: node dan penambang di komputer ini. `vps` (Android, fitur Cargo `vps`): node di VPS pengguna (src/lib/vps.ts). */
export type ModeAplikasi = 'desktop' | 'vps';

/** Mode dari aplikasi (`mode_aplikasi`); di browser `?mode=vps` memakai tiruan mode VPS. */
export async function deteksiMode(): Promise<ModeAplikasi> {
  if (diTauri()) {
    try {
      return (await invoke<string>('mode_aplikasi')) === 'vps' ? 'vps' : 'desktop';
    } catch {
      return 'desktop';
    }
  }
  if (typeof window !== 'undefined' && new URLSearchParams(window.location.search).get('mode') === 'vps') {
    return 'vps';
  }
  return 'desktop';
}

const mockInstansi: MockBackendKontrol | null = diTauri() ? null : createMockBackend();

let aktif: Backend = mockInstansi ?? createTauriBackend();

/** Mengganti backend aktif (mode VPS memasang `createVpsBackend`, src/lib/vps.ts). */
export function aturBackend(baru: Backend): void {
  aktif = baru;
}

/** Backend aktif. Semua pemanggil memakai objek ini, sehingga mode bisa dipilih saat aplikasi mulai. */
export const backend: Backend = new Proxy({} as Backend, {
  get(_sasaran, nama: string | symbol) {
    return Reflect.get(aktif, nama);
  },
});

export const simulasikanGalatNodeDev: (() => Promise<void>) | null =
  import.meta.env.DEV && mockInstansi !== null ? () => mockInstansi.simulasikanGalatNode() : null;
