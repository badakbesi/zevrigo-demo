import { useEffect, type ReactNode } from 'react';
import { ExternalLink, Loader2, Lock } from 'lucide-react';
import { useAppStore, terapkanTemaDokumen } from '../../lib/store';
import { perluKunci, useVpsStore } from '../../lib/storeVps';
import { TEKS } from '../../lib/teks';
import { ambilVpsApi, temaLokal } from '../../lib/vps';
import { KartuLog } from '../KartuLog';
import { KartuNode } from '../KartuNode';
import { KartuTambang } from '../KartuTambang';
import { Kepala } from '../Kepala';
import { Button, ErrorBox } from '../ui';
import { GalatVpsBox, PesanBerhasil, Sedang } from './Formulir';
import { KartuAkses } from './KartuAkses';
import { KartuPemeliharaan } from './KartuPemeliharaan';
import { KartuSaldo } from './KartuSaldo';
import { KartuVps } from './KartuVps';
import { LayarAmankan } from './LayarAmankan';
import { LayarAwal } from './LayarAwal';
import { LayarKunci } from './LayarKunci';
import { LayarPasang } from './LayarPasang';
import { LayarSambung } from './LayarSambung';

/** Selang pemantauan mode VPS (lewat SSH): kartu node, menambang, log; keadaan VPS; saldo. */
const SELANG_KEADAAN_MS = 3000;
const SELANG_STATUS_MS = 15000;
const SELANG_SALDO_MS = 30000;

/** Layar utama mode VPS sesudah aplikasi dibuka: kartu desktop (lewat pengendali di VPS) + saldo, VPS, pemeliharaan, akses. */
function UtamaVps() {
  const inisialisasi = useAppStore((s) => s.inisialisasi);
  const galatSambung = useAppStore((s) => s.galatSambung);
  const salinTeks = useAppStore((s) => s.salinTeks);
  const alamat = useAppStore((s) => s.pengaturan.alamatHadiah);
  const galat = useVpsStore((s) => s.galat);
  const pesan = useVpsStore((s) => s.pesan);
  const sibuk = useVpsStore((s) => s.sibuk);
  const segarkanStatus = useVpsStore((s) => s.segarkanStatus);
  const segarkanSaldo = useVpsStore((s) => s.segarkanSaldo);

  useEffect(() => {
    let berhenti: (() => void) | undefined;
    let dibatalkan = false;
    void inisialisasi(SELANG_KEADAAN_MS).then((fn) => {
      if (dibatalkan) {
        fn();
      } else {
        berhenti = fn;
      }
    });
    return () => {
      dibatalkan = true;
      if (berhenti) {
        berhenti();
      }
    };
  }, [inisialisasi]);

  useEffect(() => {
    void segarkanStatus();
    const id = setInterval(() => void segarkanStatus(), SELANG_STATUS_MS);
    return () => clearInterval(id);
  }, [segarkanStatus]);

  useEffect(() => {
    void segarkanSaldo(alamat);
    const id = setInterval(() => void segarkanSaldo(alamat), SELANG_SALDO_MS);
    return () => clearInterval(id);
  }, [segarkanSaldo, alamat]);

  return (
    <div className="space-y-3">
      {galatSambung && (
        <ErrorBox pesan={TEKS.vps.tidakTersambung} detail={galatSambung} onSalinDetail={salinTeks} />
      )}
      <GalatVpsBox galat={galat} />
      <PesanBerhasil pesan={pesan} />
      {sibuk === 'kendali' || sibuk === 'akses' ? <Sedang teks={TEKS.vps.memuat} /> : null}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3 items-stretch">
        <KartuNode />
        <KartuTambang />
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3 items-start">
        <KartuSaldo />
        <KartuVps />
      </div>
      <KartuLog />
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3 items-start">
        <KartuPemeliharaan />
        <KartuAkses />
      </div>
    </div>
  );
}

/** Aplikasi mode VPS (Android): tahap kata sandi, sambung, pasang, amankan, terkunci, lalu layar utama. */
export function AppVps() {
  const info = useVpsStore((s) => s.info);
  const sibuk = useVpsStore((s) => s.sibuk);
  const galat = useVpsStore((s) => s.galat);
  const muat = useVpsStore((s) => s.muat);
  const kunci = useVpsStore((s) => s.kunci);
  const bukaTautan = (url: string): void => {
    void ambilVpsApi().bukaTautan(url);
  };

  useEffect(() => {
    const tema = temaLokal();
    terapkanTemaDokumen(tema);
    useAppStore.setState((s) => ({ pengaturan: { ...s.pengaturan, tema } }));
    void muat();
  }, [muat]);

  // Kunci lagi sesudah aplikasi berada di latar belakang paling sedikit 1 menit (HP dikunci, pindah aplikasi).
  useEffect(() => {
    let tersembunyiSejak: number | null = null;
    const saatBerubah = (): void => {
      if (document.visibilityState === 'hidden') {
        tersembunyiSejak = Date.now();
        return;
      }
      const tahap = useVpsStore.getState().info?.tahap;
      if (perluKunci(tersembunyiSejak, Date.now()) && (tahap === 'siap' || tahap === 'amankan')) {
        void useVpsStore.getState().kunci();
      }
      tersembunyiSejak = null;
    };
    document.addEventListener('visibilitychange', saatBerubah);
    return () => document.removeEventListener('visibilitychange', saatBerubah);
  }, []);

  const tahap = info?.tahap ?? null;
  const terbuka = tahap === 'siap' || tahap === 'amankan';

  let isi: ReactNode;
  if (tahap === null) {
    isi =
      galat !== null ? (
        <div className="space-y-3">
          <GalatVpsBox galat={galat} />
          <Button variant="secondary" onClick={() => void muat()}>
            {TEKS.vps.status.segarkan}
          </Button>
        </div>
      ) : (
        <div className="flex justify-center py-12">
          <Loader2 aria-label={TEKS.vps.memuat} className="w-6 h-6 animate-spin text-emerald-600" />
        </div>
      );
  } else if (tahap === 'baru') {
    isi = <LayarAwal />;
  } else if (tahap === 'sambung') {
    isi = <LayarSambung />;
  } else if (tahap === 'pasang') {
    isi = <LayarPasang />;
  } else if (tahap === 'amankan') {
    isi = <LayarAmankan />;
  } else if (tahap === 'terkunci') {
    isi = <LayarKunci />;
  } else {
    isi = <UtamaVps />;
  }

  return (
    <div className="min-h-screen flex flex-col bg-slate-50 dark:bg-slate-950 text-slate-900 dark:text-slate-100">
      <Kepala
        aksi={
          terbuka ? (
            <Button
              variant="secondary"
              size="sm"
              disabled={sibuk !== null}
              onClick={() => void kunci()}
              aria-label={TEKS.vps.kunciAplikasi}
              title={TEKS.vps.kunciAplikasi}
            >
              <Lock className="w-4 h-4" aria-hidden="true" />
              <span>{TEKS.vps.kunci}</span>
            </Button>
          ) : null
        }
      />
      <main className="flex-1 max-w-5xl w-full mx-auto px-4 sm:px-6 py-3">{isi}</main>
      <footer className="border-t border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 py-2.5 flex flex-wrap items-center gap-4 text-xs text-slate-500 dark:text-slate-400">
          <button
            type="button"
            onClick={() => bukaTautan('https://zevrigo.online/')}
            className="inline-flex items-center gap-1 hover:text-slate-900 dark:hover:text-slate-100 cursor-pointer"
          >
            <span>{TEKS.kaki.panduan}</span>
            <ExternalLink className="w-3 h-3" aria-hidden="true" />
          </button>
          <button
            type="button"
            onClick={() => bukaTautan('https://demo-explorer.zevrigo.online')}
            className="inline-flex items-center gap-1 hover:text-slate-900 dark:hover:text-slate-100 cursor-pointer"
          >
            <span>{TEKS.kaki.explorer}</span>
            <ExternalLink className="w-3 h-3" aria-hidden="true" />
          </button>
        </div>
      </footer>
    </div>
  );
}
