import type { ReactNode } from 'react';
import { AlertTriangle, Monitor, Moon, Sun, X } from 'lucide-react';
import type { Tema } from '../lib/backend';
import { useAppStore } from '../lib/store';
import { TEKS } from '../lib/teks';
import { Badge, Button } from './ui';

function ikonTema(tema: Tema) {
  if (tema === 'terang') {
    return <Sun className="w-4 h-4" aria-hidden="true" />;
  }
  if (tema === 'gelap') {
    return <Moon className="w-4 h-4" aria-hidden="true" />;
  }
  return <Monitor className="w-4 h-4" aria-hidden="true" />;
}

function labelTema(tema: Tema): string {
  if (tema === 'terang') {
    return TEKS.tema.terang;
  }
  if (tema === 'gelap') {
    return TEKS.tema.gelap;
  }
  return TEKS.tema.sistem;
}

/** `aksi`: tombol tambahan di kanan sebelum tombol tema (mode VPS: Kunci). */
export function Kepala({ aksi = null }: { aksi?: ReactNode } = {}) {
  const info = useAppStore((s) => s.info);
  const tema = useAppStore((s) => s.pengaturan.tema);
  const putarTema = useAppStore((s) => s.putarTema);
  const spandukDitutup = useAppStore((s) => s.spandukDitutup);
  const tutupSpanduk = useAppStore((s) => s.tutupSpanduk);

  const teksTema = labelTema(tema);

  return (
    <header className="bg-white dark:bg-slate-900 border-b border-slate-200 dark:border-slate-800">
      <div className="max-w-5xl mx-auto px-4 sm:px-6 py-3 flex items-center justify-between gap-4">
        <div className="flex flex-wrap items-center gap-2.5 min-w-0">
          <h1 className="text-base sm:text-lg font-semibold tracking-tight text-slate-900 dark:text-slate-100 whitespace-nowrap">
            {TEKS.app.nama}
          </h1>
          <Badge tone="amber">{TEKS.app.lencanaDemo}</Badge>
          <span className="text-xs font-mono tabular-nums text-slate-500 dark:text-slate-400 whitespace-nowrap">
            v{info.versi}
          </span>
        </div>

        <div className="flex items-center gap-2">
        {aksi}
        <Button
          variant="secondary"
          size="sm"
          onClick={() => void putarTema()}
          aria-label={`${TEKS.tema.labelTombol} (${teksTema})`}
          title={teksTema}
        >
          {ikonTema(tema)}
          <span className="hidden sm:inline">{teksTema}</span>
        </Button>
        </div>
      </div>

      {!spandukDitutup && (
        <div
          role="region"
          aria-label={TEKS.app.lencanaDemo}
          className="bg-amber-50 dark:bg-amber-950/40 border-t border-amber-200 dark:border-amber-900/60"
        >
          <div className="max-w-5xl mx-auto px-4 sm:px-6 py-2 flex items-center justify-between gap-3">
            <div className="flex items-center gap-2 text-xs text-amber-900 dark:text-amber-200 min-w-0">
              <AlertTriangle
                aria-hidden="true"
                className="w-3.5 h-3.5 text-amber-600 dark:text-amber-400 shrink-0"
              />
              <p className="leading-snug">{TEKS.app.spandukPeringatan}</p>
            </div>
            <button
              type="button"
              onClick={tutupSpanduk}
              aria-label={TEKS.app.tutupSpanduk}
              className="p-1 rounded-md text-amber-700 hover:text-amber-950 hover:bg-amber-200/60 dark:text-amber-300 dark:hover:text-amber-100 dark:hover:bg-amber-900/60 transition-colors cursor-pointer shrink-0 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-amber-500"
            >
              <X className="w-3.5 h-3.5" aria-hidden="true" />
            </button>
          </div>
        </div>
      )}
    </header>
  );
}
