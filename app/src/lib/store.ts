import { create } from 'zustand';
import { periksaAlamatAwal } from './alamat';
import {
  backend,
  type BarisLog,
  type HasilPeriksaAlamat,
  type InfoAplikasi,
  type KeadaanNode,
  type KeadaanTambang,
  type Pengaturan,
  type Tema,
} from './backend';

const MAKS_BARIS_LOG = 500;

export type SaringLog = 'semua' | 'node' | 'penambang';

interface AppState {
  info: InfoAplikasi;
  pengaturanDimuat: boolean;
  pengaturan: Pengaturan;
  spandukDitutup: boolean;
  sedangMenutup: boolean;
  sedangProsesNode: boolean;
  sedangProsesTambang: boolean;
  hasilAlamat: HasilPeriksaAlamat;
  sedangPeriksaAlamat: boolean;
  node: KeadaanNode;
  tambang: KeadaanTambang;
  log: BarisLog[];
  waktuLogTerakhir: number | null;
  saringLog: SaringLog;

  inisialisasi: () => Promise<() => void>;
  segarkanKeadaan: () => Promise<void>;
  tutupSpanduk: () => void;
  ubahTema: (tema: Tema) => Promise<void>;
  putarTema: () => Promise<void>;
  ubahAlamatHadiah: (alamat: string) => Promise<void>;
  ubahJedaDetik: (jeda: 60 | 120 | 300) => Promise<void>;
  ubahSaringLog: (saring: SaringLog) => void;
  nyalakanNode: () => Promise<void>;
  hentikanNode: () => Promise<void>;
  mulaiTambang: () => Promise<void>;
  berhentiTambang: () => Promise<void>;
  bukaFolderData: () => Promise<void>;
  bukaTautan: (url: string) => Promise<void>;
  salinTeks: (teks: string) => Promise<void>;
}

let urutanPeriksaAlamat = 0;

function terapkanTemaDokumen(tema: Tema): void {
  if (typeof document === 'undefined') {
    return;
  }
  const preferensiGelap =
    typeof window !== 'undefined' &&
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-color-scheme: dark)').matches;
  const aktifGelap = tema === 'gelap' || (tema === 'sistem' && preferensiGelap);
  document.documentElement.classList.toggle('dark', aktifGelap);
}

export const useAppStore = create<AppState>((set, get) => ({
  info: { versi: '0.1.0', versiNode: '' },
  pengaturanDimuat: false,
  pengaturan: {
    alamatHadiah: '',
    jedaDetik: 120,
    tema: 'sistem',
  },
  spandukDitutup: false,
  sedangMenutup: false,
  sedangProsesNode: false,
  sedangProsesTambang: false,
  hasilAlamat: { sah: false, alasan: 'kosong' },
  sedangPeriksaAlamat: false,
  node: {
    status: 'mati',
    tinggiLokal: null,
    tinggiJaringan: null,
    rekan: 0,
    folderData: '',
    pesanGalat: null,
    detailGalat: null,
  },
  tambang: {
    status: 'berhenti',
    alamat: null,
    jedaDetik: 120,
    sisaJedaDetik: null,
    blokSesiIni: 0,
    blokTerakhir: [],
    pesanGalat: null,
    detailGalat: null,
  },
  log: [],
  waktuLogTerakhir: null,
  saringLog: 'semua',

  async inisialisasi(): Promise<() => void> {
    const [info, pengaturan] = await Promise.all([
      backend.info(),
      backend.muatPengaturan(),
    ]);

    terapkanTemaDokumen(pengaturan.tema);
    set({
      info,
      pengaturan,
      pengaturanDimuat: true,
    });

    if (pengaturan.alamatHadiah.trim().length > 0) {
      const cekAwal = periksaAlamatAwal(pengaturan.alamatHadiah);
      if (cekAwal.sah) {
        const hasil = await backend.periksaAlamat(pengaturan.alamatHadiah);
        set({ hasilAlamat: hasil });
      } else {
        set({ hasilAlamat: cekAwal });
      }
    }

    await get().segarkanKeadaan();

    const intervalId = setInterval(() => {
      void get().segarkanKeadaan();
    }, 1000);

    let mediaQuery: MediaQueryList | null = null;
    const tanganiMedia = (): void => {
      const temaSaatIni = get().pengaturan.tema;
      if (temaSaatIni === 'sistem') {
        terapkanTemaDokumen('sistem');
      }
    };
    if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
      mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      mediaQuery.addEventListener('change', tanganiMedia);
    }

    const berhentiTutup = backend.saatAkanTutup(() => {
      void (async () => {
        const { node, tambang } = get();
        const perluHentikan =
          node.status !== 'mati' || tambang.status !== 'berhenti';
        if (perluHentikan) {
          set({ sedangMenutup: true });
          try {
            if (tambang.status !== 'berhenti') {
              await backend.berhentiTambang();
            }
            if (node.status !== 'mati') {
              await backend.hentikanNode();
            }
          } finally {
            await backend.selesaiTutup();
            set({ sedangMenutup: false });
            await get().segarkanKeadaan();
          }
        } else {
          await backend.selesaiTutup();
        }
      })();
    });

    return () => {
      clearInterval(intervalId);
      berhentiTutup();
      if (mediaQuery !== null) {
        mediaQuery.removeEventListener('change', tanganiMedia);
      }
    };
  },

  async segarkanKeadaan(): Promise<void> {
    const sejak = get().waktuLogTerakhir;
    const [node, tambang, barisBaru] = await Promise.all([
      backend.keadaanNode(),
      backend.keadaanTambang(),
      backend.logTerbaru(sejak),
    ]);

    set((state) => {
      let logGabungan = state.log;
      let waktuTerakhir = state.waktuLogTerakhir;

      if (barisBaru.length > 0) {
        const gabungan = [...state.log, ...barisBaru];
        logGabungan =
          gabungan.length > MAKS_BARIS_LOG
            ? gabungan.slice(gabungan.length - MAKS_BARIS_LOG)
            : gabungan;
        waktuTerakhir = barisBaru[barisBaru.length - 1].waktu;
      }

      return {
        node,
        tambang,
        log: logGabungan,
        waktuLogTerakhir: waktuTerakhir,
      };
    });
  },

  tutupSpanduk(): void {
    set({ spandukDitutup: true });
  },

  async ubahTema(tema: Tema): Promise<void> {
    terapkanTemaDokumen(tema);
    const baru: Pengaturan = { ...get().pengaturan, tema };
    set({ pengaturan: baru });
    if (get().pengaturanDimuat) {
      await backend.simpanPengaturan(baru);
    }
  },

  async putarTema(): Promise<void> {
    const urutan: Tema[] = ['sistem', 'terang', 'gelap'];
    const saatIni = get().pengaturan.tema;
    const berikutnya = urutan[(urutan.indexOf(saatIni) + 1) % urutan.length];
    await get().ubahTema(berikutnya);
  },

  async ubahAlamatHadiah(alamat: string): Promise<void> {
    const baru: Pengaturan = { ...get().pengaturan, alamatHadiah: alamat };
    const cekAwal = periksaAlamatAwal(alamat);
    const idPermintaan = ++urutanPeriksaAlamat;

    set({
      pengaturan: baru,
      hasilAlamat: cekAwal,
      sedangPeriksaAlamat: cekAwal.sah,
    });

    if (get().pengaturanDimuat) {
      void backend.simpanPengaturan(baru);
    }

    if (cekAwal.sah) {
      const hasilBackend = await backend.periksaAlamat(alamat);
      if (idPermintaan === urutanPeriksaAlamat) {
        set({
          hasilAlamat: hasilBackend,
          sedangPeriksaAlamat: false,
        });
      }
    }
  },

  async ubahJedaDetik(jedaDetik: 60 | 120 | 300): Promise<void> {
    const baru: Pengaturan = { ...get().pengaturan, jedaDetik };
    set({ pengaturan: baru });
    if (get().pengaturanDimuat) {
      await backend.simpanPengaturan(baru);
    }
  },

  ubahSaringLog(saringLog: SaringLog): void {
    set({ saringLog });
  },

  async nyalakanNode(): Promise<void> {
    set({ sedangProsesNode: true });
    try {
      await backend.nyalakanNode();
      await get().segarkanKeadaan();
    } finally {
      set({ sedangProsesNode: false });
    }
  },

  async hentikanNode(): Promise<void> {
    set({ sedangProsesNode: true });
    try {
      if (get().tambang.status !== 'berhenti') {
        await backend.berhentiTambang();
      }
      await backend.hentikanNode();
      await get().segarkanKeadaan();
    } finally {
      set({ sedangProsesNode: false });
    }
  },

  async mulaiTambang(): Promise<void> {
    const { pengaturan } = get();
    set({ sedangProsesTambang: true });
    try {
      await backend.mulaiTambang(pengaturan.alamatHadiah, pengaturan.jedaDetik);
      await get().segarkanKeadaan();
    } catch {
      await get().segarkanKeadaan();
    } finally {
      set({ sedangProsesTambang: false });
    }
  },

  async berhentiTambang(): Promise<void> {
    set({ sedangProsesTambang: true });
    try {
      await backend.berhentiTambang();
      await get().segarkanKeadaan();
    } finally {
      set({ sedangProsesTambang: false });
    }
  },

  async bukaFolderData(): Promise<void> {
    await backend.bukaFolderData();
    await get().segarkanKeadaan();
  },

  async bukaTautan(url: string): Promise<void> {
    await backend.bukaTautan(url);
    await get().segarkanKeadaan();
  },

  async salinTeks(teks: string): Promise<void> {
    await backend.salinTeks(teks);
    await get().segarkanKeadaan();
  },
}));
