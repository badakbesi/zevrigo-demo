import { useEffect } from 'react';
import { ExternalLink, Loader2 } from 'lucide-react';
import { KartuLog } from './components/KartuLog';
import { KartuNode } from './components/KartuNode';
import { KartuTambang } from './components/KartuTambang';
import { Kepala } from './components/Kepala';
import { simulasikanGalatNodeDev } from './lib/backend';
import { useAppStore } from './lib/store';
import { TEKS } from './lib/teks';

export default function App() {
  const inisialisasi = useAppStore((s) => s.inisialisasi);
  const sedangMenutup = useAppStore((s) => s.sedangMenutup);
  const bukaTautan = useAppStore((s) => s.bukaTautan);
  const segarkanKeadaan = useAppStore((s) => s.segarkanKeadaan);

  useEffect(() => {
    let berhenti: (() => void) | undefined;
    let dibatalkan = false;

    void inisialisasi().then((fn) => {
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

  async function tanganiSimulasiGalat(): Promise<void> {
    if (!simulasikanGalatNodeDev) {
      return;
    }
    await simulasikanGalatNodeDev();
    await segarkanKeadaan();
  }

  return (
    <div className="min-h-screen flex flex-col bg-slate-50 dark:bg-slate-950 text-slate-900 dark:text-slate-100">
      <Kepala />

      <main className="flex-1 max-w-5xl w-full mx-auto px-4 sm:px-6 py-3 space-y-3">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 items-stretch">
          <KartuNode />
          <KartuTambang />
        </div>
        <KartuLog />
      </main>

      <footer className="border-t border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900">
        <div className="max-w-5xl mx-auto px-4 sm:px-6 py-2.5 flex flex-wrap items-center justify-between gap-3 text-xs text-slate-500 dark:text-slate-400">
          <div className="flex items-center gap-4">
            <button
              type="button"
              onClick={() => void bukaTautan('https://zevrigo.online/')}
              className="inline-flex items-center gap-1 hover:text-slate-900 dark:hover:text-slate-100 transition-colors cursor-pointer focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-emerald-500 rounded"
            >
              <span>{TEKS.kaki.panduan}</span>
              <ExternalLink className="w-3 h-3" aria-hidden="true" />
            </button>
            <button
              type="button"
              onClick={() => void bukaTautan('https://demo-explorer.zevrigo.online')}
              className="inline-flex items-center gap-1 hover:text-slate-900 dark:hover:text-slate-100 transition-colors cursor-pointer focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-emerald-500 rounded"
            >
              <span>{TEKS.kaki.explorer}</span>
              <ExternalLink className="w-3 h-3" aria-hidden="true" />
            </button>
          </div>

          {import.meta.env.DEV && simulasikanGalatNodeDev && (
            <button
              type="button"
              onClick={() => void tanganiSimulasiGalat()}
              className="text-[11px] text-rose-600 hover:text-rose-500 dark:text-rose-400 dark:hover:text-rose-300 hover:underline cursor-pointer focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-rose-500 rounded"
            >
              {TEKS.kaki.simulasiGalat}
            </button>
          )}
        </div>
      </footer>

      {sedangMenutup && (
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby="judul-sedang-menutup"
          className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/70 backdrop-blur-xs p-4"
        >
          <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-6 max-w-sm w-full text-center space-y-3 shadow-lg">
            <Loader2
              aria-hidden="true"
              className="w-7 h-7 text-emerald-600 dark:text-emerald-400 animate-spin mx-auto"
            />
            <p
              id="judul-sedang-menutup"
              className="text-sm font-semibold text-slate-900 dark:text-slate-100"
            >
              {TEKS.app.menutupAplikasi}
            </p>
            <p className="text-xs text-slate-500 dark:text-slate-400">
              {TEKS.app.menutupAplikasiSub}
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
