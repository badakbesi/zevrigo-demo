import type {
  Backend,
  BarisLog,
  BlokDitemukan,
  HasilPeriksaAlamat,
  InfoAplikasi,
  KeadaanNode,
  KeadaanTambang,
  Pengaturan,
} from './backend';
import { formatAngka } from './format';

const KARAKTER_BECH32 = /^[qpzry9x8gf2tvdw0s3jn54khce6mua7l]+$/;
const PANJANG_ALAMAT_SAH = 65;
const MAKS_LOG = 500;
const MAKS_BLOK_TERAKHIR = 5;
const TINGGI_JARINGAN_AWAL = 1943;

function buatIdBlokAcak(): string {
  const hex = '0123456789abcdef';
  let hasil = '';
  for (let i = 0; i < 64; i += 1) {
    hasil += hex[Math.floor(Math.random() * 16)];
  }
  return hasil;
}

export interface MockBackendKontrol extends Backend {
  simulasikanGalatNode(): Promise<void>;
  picuAkanTutup(): void;
}

export function createMockBackend(): MockBackendKontrol {
  const infoAplikasi: InfoAplikasi = {
    versi: '0.1.0',
    versiNode: 'zevrigo-node 0.9.4-regtest',
  };

  let pengaturan: Pengaturan = {
    alamatHadiah: '',
    jedaDetik: 120,
    tema: 'sistem',
  };

  let node: KeadaanNode = {
    status: 'mati',
    tinggiLokal: null,
    tinggiJaringan: null,
    rekan: 0,
    folderData: '/home/peserta/.zevrigo-demo/regtest',
    pesanGalat: null,
    detailGalat: null,
  };

  let tambang: KeadaanTambang = {
    status: 'berhenti',
    alamat: null,
    jedaDetik: 120,
    sisaJedaDetik: null,
    blokSesiIni: 0,
    blokTerakhir: [],
    pesanGalat: null,
    detailGalat: null,
  };

  const logBuffer: BarisLog[] = [];
  let waktuLogTerakhir = 0;

  const pendengarTutup = new Set<() => void>();
  const timerNode = new Set<ReturnType<typeof setTimeout>>();
  let intervalSinkron: ReturnType<typeof setInterval> | null = null;
  let intervalJaringan: ReturnType<typeof setInterval> | null = null;
  let timerTambang: ReturnType<typeof setTimeout> | null = null;
  let intervalJeda: ReturnType<typeof setInterval> | null = null;

  function tambahLog(
    sumber: BarisLog['sumber'],
    tingkat: BarisLog['tingkat'],
    teks: string,
  ): void {
    const sekarang = Date.now();
    const waktu = sekarang <= waktuLogTerakhir ? waktuLogTerakhir + 1 : sekarang;
    waktuLogTerakhir = waktu;
    logBuffer.push({ waktu, sumber, tingkat, teks });
    if (logBuffer.length > MAKS_LOG) {
      logBuffer.splice(0, logBuffer.length - MAKS_LOG);
    }
  }

  function bersihkanTimerNode(): void {
    for (const t of timerNode) {
      clearTimeout(t);
    }
    timerNode.clear();
    if (intervalSinkron !== null) {
      clearInterval(intervalSinkron);
      intervalSinkron = null;
    }
    if (intervalJaringan !== null) {
      clearInterval(intervalJaringan);
      intervalJaringan = null;
    }
  }

  function bersihkanTimerTambang(): void {
    if (timerTambang !== null) {
      clearTimeout(timerTambang);
      timerTambang = null;
    }
    if (intervalJeda !== null) {
      clearInterval(intervalJeda);
      intervalJeda = null;
    }
  }

  function jalankanSiklusTambang(): void {
    bersihkanTimerTambang();
    if (node.status !== 'sinkron' || tambang.alamat === null) {
      return;
    }

    tambang = {
      ...tambang,
      status: 'menambang',
      sisaJedaDetik: null,
    };
    tambahLog('penambang', 'info', 'Sedang mencari blok baru...');

    const durasiTambangMs = 3000;
    timerTambang = setTimeout(() => {
      timerTambang = null;
      if (node.status !== 'sinkron' || tambang.status !== 'menambang') {
        return;
      }

      const tinggiBaru = (node.tinggiJaringan ?? TINGGI_JARINGAN_AWAL) + 1;
      const idBlok = buatIdBlokAcak();
      const blokBaru: BlokDitemukan = {
        tinggi: tinggiBaru,
        idBlok,
        waktu: Date.now(),
      };

      node = {
        ...node,
        tinggiJaringan: tinggiBaru,
        tinggiLokal: tinggiBaru,
      };

      const daftarBaru = [blokBaru, ...tambang.blokTerakhir].slice(0, MAKS_BLOK_TERAKHIR);
      tambang = {
        ...tambang,
        status: 'menunggu',
        sisaJedaDetik: tambang.jedaDetik,
        blokSesiIni: tambang.blokSesiIni + 1,
        blokTerakhir: daftarBaru,
      };

      tambahLog(
        'penambang',
        'info',
        `Blok ditemukan pada tinggi ${formatAngka(tinggiBaru)} (${idBlok.slice(0, 8)}...)`,
      );
      tambahLog(
        'node',
        'info',
        `Blok diterima: tinggi ${formatAngka(tinggiBaru)}`,
      );

      // Jeda dipercepat 10x pada tiruan agar mudah diuji
      intervalJeda = setInterval(() => {
        if (tambang.status !== 'menunggu' || tambang.sisaJedaDetik === null) {
          bersihkanTimerTambang();
          return;
        }
        const sisaBerikut = Math.max(0, tambang.sisaJedaDetik - 10);
        if (sisaBerikut === 0) {
          bersihkanTimerTambang();
          jalankanSiklusTambang();
        } else {
          tambang = {
            ...tambang,
            sisaJedaDetik: sisaBerikut,
          };
        }
      }, 1000);
    }, durasiTambangMs);
  }

  function mulaiPemantauanJaringan(): void {
    if (intervalJaringan !== null) {
      clearInterval(intervalJaringan);
    }
    intervalJaringan = setInterval(() => {
      if (node.status !== 'sinkron' || node.tinggiJaringan === null) {
        return;
      }
      const tinggiBaru = node.tinggiJaringan + 1;
      const rekanAcak = Math.random() > 0.65 ? 3 : 2;
      node = {
        ...node,
        tinggiJaringan: tinggiBaru,
        tinggiLokal: tinggiBaru,
        rekan: rekanAcak,
      };
      tambahLog('node', 'info', `Blok diterima: tinggi ${formatAngka(tinggiBaru)}`);
    }, 30000);
  }

  async function validasiAlamat(masukan: string): Promise<HasilPeriksaAlamat> {
    const alamat = masukan.trim();
    if (alamat.length === 0) {
      return { sah: false, alasan: 'kosong' };
    }
    const hurufKecil = alamat.toLowerCase();
    if (hurufKecil.startsWith('tzvrg1') || hurufKecil.startsWith('zvrg1')) {
      return { sah: false, alasan: 'jaringan-lain' };
    }
    if (
      alamat !== hurufKecil ||
      !alamat.startsWith('rzvrg1') ||
      alamat.length !== PANJANG_ALAMAT_SAH ||
      !KARAKTER_BECH32.test(alamat.slice(6))
    ) {
      return { sah: false, alasan: 'format' };
    }
    return { sah: true, alasan: null };
  }

  return {
    async info(): Promise<InfoAplikasi> {
      return { ...infoAplikasi };
    },

    async muatPengaturan(): Promise<Pengaturan> {
      return { ...pengaturan };
    },

    async simpanPengaturan(p: Pengaturan): Promise<void> {
      pengaturan = { ...p };
    },

    async nyalakanNode(): Promise<void> {
      if (node.status !== 'mati' && node.status !== 'galat') {
        return;
      }
      bersihkanTimerNode();

      node = {
        ...node,
        status: 'menyala',
        tinggiLokal: null,
        tinggiJaringan: null,
        rekan: 0,
        pesanGalat: null,
        detailGalat: null,
      };
      tambahLog('node', 'info', 'Menyalakan node Zevrigo (jaringan demo regtest)...');

      const t1 = setTimeout(() => {
        node = {
          ...node,
          status: 'menyambung',
          rekan: 1,
          tinggiLokal: 0,
          tinggiJaringan: TINGGI_JARINGAN_AWAL,
        };
        tambahLog('node', 'info', 'Seed resmi bawaan dipakai: demo-seed-p2p.zevrigo.online');
        tambahLog('node', 'info', 'Rekan terhubung (1 rekan)');

        const t2 = setTimeout(() => {
          node = {
            ...node,
            status: 'mengejar',
            rekan: 2,
          };
          tambahLog(
            'node',
            'info',
            `Mengejar chain dari blok 0 ke ${formatAngka(TINGGI_JARINGAN_AWAL)}...`,
          );

          let langkah = 0;
          const totalLangkah = 10;
          intervalSinkron = setInterval(() => {
            langkah += 1;
            const target = node.tinggiJaringan ?? TINGGI_JARINGAN_AWAL;
            const tinggiBaru = Math.min(
              target,
              Math.round((target * langkah) / totalLangkah),
            );

            if (langkah >= totalLangkah || tinggiBaru >= target) {
              if (intervalSinkron !== null) {
                clearInterval(intervalSinkron);
                intervalSinkron = null;
              }
              node = {
                ...node,
                status: 'sinkron',
                tinggiLokal: target,
                tinggiJaringan: target,
                rekan: 2,
              };
              tambahLog('node', 'info', `Blok diterima: tinggi ${formatAngka(target)}`);
              tambahLog('node', 'info', `Node sinkron pada tinggi ${formatAngka(target)}`);
              mulaiPemantauanJaringan();
            } else {
              node = {
                ...node,
                tinggiLokal: tinggiBaru,
                rekan: langkah >= 5 ? 3 : 2,
              };
              if (langkah % 2 === 0) {
                tambahLog('node', 'info', `Blok diterima: tinggi ${formatAngka(tinggiBaru)}`);
              }
            }
          }, 1000);
        }, 2000);
        timerNode.add(t2);
      }, 1000);
      timerNode.add(t1);
    },

    async hentikanNode(): Promise<void> {
      bersihkanTimerTambang();
      bersihkanTimerNode();

      if (tambang.status !== 'berhenti') {
        tambang = {
          ...tambang,
          status: 'berhenti',
          sisaJedaDetik: null,
        };
        tambahLog('penambang', 'info', 'Penambang dihentikan sebelum node dimatikan.');
      }

      node = {
        ...node,
        status: 'mati',
        tinggiLokal: null,
        tinggiJaringan: null,
        rekan: 0,
        pesanGalat: null,
        detailGalat: null,
      };
      tambahLog('node', 'info', 'Node dihentikan.');
    },

    async keadaanNode(): Promise<KeadaanNode> {
      return { ...node };
    },

    periksaAlamat: validasiAlamat,

    async mulaiTambang(alamat: string, jedaDetik: number): Promise<void> {
      if (node.status !== 'sinkron') {
        const pesan = 'Node harus dalam keadaan Sinkron sebelum mulai menambang.';
        tambang = {
          ...tambang,
          pesanGalat: pesan,
          detailGalat: `Status node saat ini: ${node.status}`,
        };
        tambahLog('penambang', 'galat', pesan);
        throw new Error(pesan);
      }

      const cek = await validasiAlamat(alamat);
      if (!cek.sah) {
        const pesan = 'Alamat hadiah tidak sah untuk jaringan demo.';
        tambang = {
          ...tambang,
          pesanGalat: pesan,
          detailGalat: `Alasan validasi: ${cek.alasan ?? 'tidak diketahui'}`,
        };
        tambahLog('penambang', 'galat', pesan);
        throw new Error(pesan);
      }

      const alamatBersih = alamat.trim();
      tambang = {
        ...tambang,
        alamat: alamatBersih,
        jedaDetik,
        pesanGalat: null,
        detailGalat: null,
      };
      tambahLog(
        'penambang',
        'info',
        `Memulai penambang ke alamat ${alamatBersih.slice(0, 12)}... (jeda ${jedaDetik} detik)`,
      );
      jalankanSiklusTambang();
    },

    async berhentiTambang(): Promise<void> {
      bersihkanTimerTambang();
      if (tambang.status !== 'berhenti') {
        tambang = {
          ...tambang,
          status: 'berhenti',
          sisaJedaDetik: null,
        };
        tambahLog('penambang', 'info', 'Penambang dihentikan oleh pengguna.');
      }
    },

    async keadaanTambang(): Promise<KeadaanTambang> {
      return {
        ...tambang,
        blokTerakhir: tambang.blokTerakhir.map((b) => ({ ...b })),
      };
    },

    async logTerbaru(sejakWaktu: number | null): Promise<BarisLog[]> {
      if (sejakWaktu === null) {
        return logBuffer.map((b) => ({ ...b }));
      }
      return logBuffer
        .filter((b) => b.waktu > sejakWaktu)
        .map((b) => ({ ...b }));
    },

    async bukaFolderData(): Promise<void> {
      tambahLog('aplikasi', 'info', `Membuka folder data: ${node.folderData}`);
    },

    async bukaTautan(url: string): Promise<void> {
      tambahLog('aplikasi', 'info', `Membuka tautan eksternal: ${url}`);
    },

    async salinTeks(teks: string): Promise<void> {
      if (typeof navigator !== 'undefined' && navigator.clipboard?.writeText) {
        try {
          await navigator.clipboard.writeText(teks);
        } catch {
          // Abaikan jika izin clipboard dibatasi lingkungan uji
        }
      }
      tambahLog('aplikasi', 'info', `Menyalin ${teks.length} karakter ke papan klip.`);
    },

    saatAkanTutup(cb: () => void): () => void {
      pendengarTutup.add(cb);
      return () => {
        pendengarTutup.delete(cb);
      };
    },

    async selesaiTutup(): Promise<void> {
      tambahLog('aplikasi', 'info', 'Penutupan aman selesai.');
    },

    async simulasikanGalatNode(): Promise<void> {
      bersihkanTimerTambang();
      bersihkanTimerNode();
      if (tambang.status !== 'berhenti') {
        tambang = {
          ...tambang,
          status: 'berhenti',
          sisaJedaDetik: null,
        };
        tambahLog('penambang', 'peringatan', 'Penambang berhenti karena node mengalami galat.');
      }
      node = {
        ...node,
        status: 'galat',
        rekan: 0,
        pesanGalat: 'Koneksi ke seed jaringan demo terputus.',
        detailGalat:
          'ERR_P2P_SEED_UNREACHABLE: gagal tersambung ke demo-seed-p2p.zevrigo.online:18444 (waktu habis 15000 ms)',
      };
      tambahLog('node', 'galat', 'Galat node: koneksi ke seed jaringan demo terputus.');
    },

    picuAkanTutup(): void {
      for (const cb of pendengarTutup) {
        cb();
      }
    },
  };
}
