import { create } from 'zustand';
import { TEKS } from './teks';
import {
  ambilVpsApi,
  keGalat,
  type GalatVps,
  type InfoVps,
  type PerintahVps,
  type SaldoVps,
  type StatusVps,
} from './vps';
import { periksaAlamatAwal } from './alamat';

/** Aplikasi terkunci lagi sesudah berada di latar belakang selama ini (HP dikunci, pindah aplikasi; pola wallet). */
export const KUNCI_SESUDAH_MS = 60_000;

export function perluKunci(tersembunyiSejak: number | null, sekarang: number): boolean {
  return tersembunyiSejak !== null && sekarang - tersembunyiSejak >= KUNCI_SESUDAH_MS;
}

export type SibukVps =
  | 'muat'
  | 'sandi'
  | 'impor'
  | 'buka'
  | 'sambung'
  | 'pasang'
  | 'amankan'
  | 'kendali'
  | 'akses'
  | 'ekspor';

interface VpsState {
  info: InfoVps | null;
  sibuk: SibukVps | null;
  galat: GalatVps | null;
  pesan: string | null;
  status: StatusVps | null;
  galatStatus: string | null;
  saldo: SaldoVps | null;
  teksEkspor: string | null;

  muat: () => Promise<void>;
  bersihkanGalat: () => void;
  buatSandi: (sandi: string, ulang: string) => Promise<void>;
  impor: (teks: string, sandi: string) => Promise<void>;
  buka: (sandi: string) => Promise<void>;
  kunci: () => Promise<void>;
  sambung: (host: string, port: number, pengguna: string, sandi: string) => Promise<void>;
  batal: () => Promise<void>;
  pasang: () => Promise<void>;
  amankan: () => Promise<void>;
  lewatiAmankan: () => Promise<void>;
  segarkanStatus: () => Promise<void>;
  segarkanSaldo: (alamat: string) => Promise<void>;
  kendali: (perintah: PerintahVps, pesanBerhasil: string) => Promise<boolean>;
  gantiToken: () => Promise<void>;
  ekspor: () => Promise<void>;
  tutupEkspor: () => void;
  lupakan: () => Promise<void>;
  cabutAkses: () => Promise<void>;
  copot: (hapus: boolean) => Promise<void>;
}

const KOSONG = { status: null, galatStatus: null, saldo: null, teksEkspor: null } as const;

export const useVpsStore = create<VpsState>((set, get) => {
  /** Menjalankan satu langkah: `sibuk` selama berjalan, galat ditampilkan, keadaan baru disimpan. */
  async function jalankan(
    sibuk: SibukVps,
    kerja: () => Promise<InfoVps | null>,
  ): Promise<boolean> {
    set({ sibuk, galat: null, pesan: null });
    try {
      const hasil = await kerja();
      if (hasil !== null) {
        set({ info: hasil });
      }
      return true;
    } catch (e) {
      set({ galat: keGalat(e) });
      return false;
    } finally {
      set({ sibuk: null });
    }
  }

  const api = () => ambilVpsApi();

  return {
    info: null,
    sibuk: null,
    galat: null,
    pesan: null,
    status: null,
    galatStatus: null,
    saldo: null,
    teksEkspor: null,

    async muat() {
      await jalankan('muat', () => api().keadaan());
    },

    bersihkanGalat() {
      set({ galat: null, pesan: null });
    },

    async buatSandi(sandi: string, ulang: string) {
      if (sandi !== ulang) {
        set({ galat: { kode: 'masukan', pesan: TEKS.vps.sandiBeda } });
        return;
      }
      await jalankan('sandi', () => api().buatSandi(sandi));
    },

    async impor(teks: string, sandi: string) {
      await jalankan('impor', () => api().impor(teks.trim(), sandi));
    },

    async buka(sandi: string) {
      await jalankan('buka', () => api().buka(sandi));
    },

    async kunci() {
      set({ ...KOSONG });
      await jalankan('buka', () => api().kunci());
    },

    async sambung(host: string, port: number, pengguna: string, sandi: string) {
      await jalankan('sambung', () => api().sambung(host.trim(), port, pengguna.trim(), sandi));
    },

    async batal() {
      await jalankan('sambung', () => api().batal());
    },

    async pasang() {
      await jalankan('pasang', () => api().pasang());
    },

    async amankan() {
      await jalankan('amankan', () => api().amankan());
    },

    async lewatiAmankan() {
      await jalankan('amankan', () => api().lewatiAmankan());
    },

    async segarkanStatus() {
      try {
        set({ status: await api().status(), galatStatus: null });
      } catch (e) {
        set({ galatStatus: keGalat(e).pesan });
      }
    },

    async segarkanSaldo(alamat: string) {
      if (!periksaAlamatAwal(alamat).sah) {
        set({ saldo: null });
        return;
      }
      try {
        set({ saldo: await api().rpc<SaldoVps>('saldo', { alamat: alamat.trim() }) });
      } catch {
        set({ saldo: null });
      }
    },

    async kendali(perintah: PerintahVps, pesanBerhasil: string) {
      const ok = await jalankan('kendali', async () => {
        await api().kendali(perintah);
        set({ pesan: pesanBerhasil });
        return null;
      });
      if (ok && perintah.jenis !== 'restart_vps') {
        await get().segarkanStatus();
      }
      return ok;
    },

    async gantiToken() {
      await jalankan('kendali', async () => {
        await api().gantiToken();
        set({ pesan: TEKS.vps.pemeliharaan.tokenDiganti });
        return null;
      });
    },

    async ekspor() {
      await jalankan('ekspor', async () => {
        set({ teksEkspor: await api().ekspor() });
        return null;
      });
    },

    tutupEkspor() {
      set({ teksEkspor: null });
    },

    async lupakan() {
      set({ ...KOSONG });
      await jalankan('akses', () => api().lupakan());
    },

    async cabutAkses() {
      const ok = await jalankan('akses', () => api().cabutAkses());
      if (ok) {
        set({ ...KOSONG });
      }
    },

    async copot(hapus: boolean) {
      const ok = await jalankan('akses', () => api().copot(hapus));
      if (ok) {
        set({ ...KOSONG });
      }
    },
  };
});
