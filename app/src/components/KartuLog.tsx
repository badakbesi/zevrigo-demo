import { useEffect, useMemo, useRef, useState } from 'react';
import { ArrowDown, Check, Copy, ExternalLink } from 'lucide-react';
import { formatWaktu } from '../lib/format';
import { type SaringLog, useAppStore } from '../lib/store';
import { TEKS } from '../lib/teks';
import { Button, Card } from './ui';

const OPSI_SARING: Array<{ nilai: SaringLog; label: string }> = [
  { nilai: 'semua', label: TEKS.log.saring.semua },
  { nilai: 'node', label: TEKS.log.saring.node },
  { nilai: 'penambang', label: TEKS.log.saring.penambang },
];

export function KartuLog() {
  const log = useAppStore((s) => s.log);
  const saringLog = useAppStore((s) => s.saringLog);
  const ubahSaringLog = useAppStore((s) => s.ubahSaringLog);
  const salinTeks = useAppStore((s) => s.salinTeks);
  const bukaTautan = useAppStore((s) => s.bukaTautan);

  const wadahRef = useRef<HTMLDivElement | null>(null);
  const [diBawah, setDiBawah] = useState(true);
  const [tersalin, setTersalin] = useState(false);

  const logTersaring = useMemo(() => {
    if (saringLog === 'semua') {
      return log;
    }
    return log.filter((b) => b.sumber === saringLog);
  }, [log, saringLog]);

  useEffect(() => {
    const el = wadahRef.current;
    if (!el || !diBawah) {
      return;
    }
    el.scrollTop = el.scrollHeight;
  }, [logTersaring, diBawah]);

  function tanganiGulir(): void {
    const el = wadahRef.current;
    if (!el) {
      return;
    }
    const jarakDariBawah = el.scrollHeight - el.scrollTop - el.clientHeight;
    setDiBawah(jarakDariBawah <= 24);
  }

  function lompatKeBawah(): void {
    const el = wadahRef.current;
    if (!el) {
      return;
    }
    el.scrollTop = el.scrollHeight;
    setDiBawah(true);
  }

  async function tanganiSalinLog(): Promise<void> {
    const teks = logTersaring
      .map(
        (b) =>
          `${formatWaktu(b.waktu)} [${TEKS.log.sumber[b.sumber]}] ${b.teks}`,
      )
      .join('\n');
    await salinTeks(teks);
    setTersalin(true);
    setTimeout(() => setTersalin(false), 1800);
  }

  return (
    <Card className="space-y-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap items-center gap-3">
          <h2 className="text-base font-semibold text-slate-900 dark:text-slate-100">
            {TEKS.log.judul}
          </h2>
          <div
            role="group"
            aria-label={TEKS.log.judul}
            className="inline-flex items-center gap-1 p-1 rounded-lg bg-slate-100 dark:bg-slate-800/90"
          >
            {OPSI_SARING.map((opsi) => {
              const aktif = saringLog === opsi.nilai;
              return (
                <button
                  key={opsi.nilai}
                  type="button"
                  aria-pressed={aktif}
                  onClick={() => ubahSaringLog(opsi.nilai)}
                  className={`px-2.5 py-1 text-xs font-medium rounded-md transition-colors whitespace-nowrap shrink-0 cursor-pointer focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-emerald-500 ${
                    aktif
                      ? 'bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100 shadow-xs'
                      : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-slate-100'
                  }`}
                >
                  {opsi.label}
                </button>
              );
            })}
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {!diBawah && logTersaring.length > 0 && (
            <Button variant="ghost" size="sm" onClick={lompatKeBawah}>
              <ArrowDown className="w-3.5 h-3.5" aria-hidden="true" />
              <span>{TEKS.log.lompatKeBawah}</span>
            </Button>
          )}
          <Button
            variant="secondary"
            size="sm"
            disabled={logTersaring.length === 0}
            onClick={() => void tanganiSalinLog()}
          >
            {tersalin ? (
              <Check className="w-3.5 h-3.5 text-emerald-600" aria-hidden="true" />
            ) : (
              <Copy className="w-3.5 h-3.5" aria-hidden="true" />
            )}
            <span>{tersalin ? TEKS.log.tersalin : TEKS.log.salinLog}</span>
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() =>
              void bukaTautan('https://github.com/badakbesi/zevrigo-demo/issues')
            }
          >
            <span>{TEKS.log.laporkanMasalah}</span>
            <ExternalLink className="w-3.5 h-3.5" aria-hidden="true" />
          </Button>
        </div>
      </div>

      <div
        ref={wadahRef}
        onScroll={tanganiGulir}
        tabIndex={0}
        aria-label={TEKS.log.judul}
        className="h-36 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950 p-3 overflow-y-auto font-mono text-xs tabular-nums leading-relaxed focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-emerald-500"
      >
        {logTersaring.length === 0 ? (
          <p className="text-slate-500 dark:text-slate-400 font-sans">
            {TEKS.log.kosong}
          </p>
        ) : (
          <ul className="space-y-1">
            {logTersaring.map((baris, idx) => {
              const warnaTeks =
                baris.tingkat === 'galat'
                  ? 'text-rose-700 dark:text-rose-300'
                  : baris.tingkat === 'peringatan'
                    ? 'text-amber-700 dark:text-amber-300'
                    : 'text-slate-700 dark:text-slate-300';
              return (
                <li key={`${baris.waktu}-${idx}`} className={`flex gap-2 ${warnaTeks}`}>
                  <span className="text-slate-400 dark:text-slate-500 shrink-0">
                    {formatWaktu(baris.waktu)}
                  </span>
                  <span className="text-slate-500 dark:text-slate-400 shrink-0 w-16">
                    [{TEKS.log.sumber[baris.sumber]}]
                  </span>
                  <span className="break-words min-w-0">{baris.teks}</span>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </Card>
  );
}
