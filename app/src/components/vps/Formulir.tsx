import type { InputHTMLAttributes, ReactNode } from 'react';
import { CheckCircle2, Loader2 } from 'lucide-react';
import type { GalatVps } from '../../lib/vps';
import { useAppStore } from '../../lib/store';
import { TEKS } from '../../lib/teks';
import { Button, ErrorBox } from '../ui';

const KELAS_ISIAN =
  'w-full rounded-lg border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-950 px-3 py-2 text-sm text-slate-900 dark:text-slate-100 placeholder:text-slate-400 dark:placeholder:text-slate-500 focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-emerald-500 disabled:opacity-60';

interface IsianProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'onChange'> {
  id: string;
  label: string;
  nilai: string;
  ubah: (nilai: string) => void;
}

/** Isian berlabel (gaya kartu aplikasi). */
export function Isian({ id, label, nilai, ubah, className = '', ...rest }: IsianProps) {
  return (
    <div className={`space-y-1 ${className}`}>
      <label htmlFor={id} className="block text-xs font-medium text-slate-600 dark:text-slate-300">
        {label}
      </label>
      <input
        id={id}
        value={nilai}
        onChange={(e) => ubah(e.target.value)}
        className={KELAS_ISIAN}
        {...rest}
      />
    </div>
  );
}

/** Area teks berlabel (ekspor dan impor data VPS). */
export function AreaTeks({
  id,
  label,
  nilai,
  ubah,
  bacaSaja = false,
}: {
  id: string;
  label: string;
  nilai: string;
  ubah?: (nilai: string) => void;
  bacaSaja?: boolean;
}) {
  return (
    <div className="space-y-1">
      <label htmlFor={id} className="block text-xs font-medium text-slate-600 dark:text-slate-300">
        {label}
      </label>
      <textarea
        id={id}
        value={nilai}
        readOnly={bacaSaja}
        onChange={(e) => ubah?.(e.target.value)}
        rows={5}
        spellCheck={false}
        autoCapitalize="off"
        autoCorrect="off"
        className={`${KELAS_ISIAN} font-mono text-xs [overflow-wrap:anywhere]`}
      />
    </div>
  );
}

/** Galat langkah VPS (pesan + kode di detail teknis). */
export function GalatVpsBox({ galat }: { galat: GalatVps | null }) {
  const salinTeks = useAppStore((s) => s.salinTeks);
  if (galat === null) {
    return null;
  }
  return <ErrorBox pesan={galat.pesan} detail={`kode: ${galat.kode}`} onSalinDetail={salinTeks} />;
}

/** Pesan berhasil. */
export function PesanBerhasil({ pesan }: { pesan: string | null }) {
  if (pesan === null) {
    return null;
  }
  return (
    <div
      role="status"
      className="flex items-start gap-2 rounded-lg border border-emerald-200 dark:border-emerald-900/70 bg-emerald-50 dark:bg-emerald-950/40 p-3 text-sm text-emerald-900 dark:text-emerald-100"
    >
      <CheckCircle2 aria-hidden="true" className="w-4 h-4 mt-0.5 shrink-0 text-emerald-600 dark:text-emerald-400" />
      <p className="leading-snug">{pesan}</p>
    </div>
  );
}

/** Teks sedang bekerja dengan ikon berputar. */
export function Sedang({ teks }: { teks: string }) {
  return (
    <p role="status" className="flex items-start gap-2 text-sm text-slate-600 dark:text-slate-300">
      <Loader2 aria-hidden="true" className="w-4 h-4 mt-0.5 shrink-0 animate-spin text-emerald-600 dark:text-emerald-400" />
      <span className="leading-snug">{teks}</span>
    </p>
  );
}

/** Kotak konfirmasi di dalam kartu (pola konfirmasi KartuNode). */
export function Konfirmasi({
  id,
  judul,
  isi,
  labelYa = TEKS.vps.pemeliharaan.ya,
  bahaya = false,
  sibuk = false,
  onYa,
  onBatal,
  children,
}: {
  id: string;
  judul: string;
  isi: string;
  labelYa?: string;
  bahaya?: boolean;
  sibuk?: boolean;
  onYa: () => void;
  onBatal: () => void;
  children?: ReactNode;
}) {
  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby={id}
      className="rounded-lg border border-amber-300 dark:border-amber-800 bg-amber-50 dark:bg-amber-950/50 p-3.5 space-y-3"
    >
      <div className="space-y-1">
        <p id={id} className="text-sm font-semibold text-amber-950 dark:text-amber-100">
          {judul}
        </p>
        <p className="text-xs text-amber-900 dark:text-amber-200 leading-relaxed">{isi}</p>
      </div>
      {children}
      <div className="flex flex-wrap items-center gap-2">
        <Button variant={bahaya ? 'danger' : 'primary'} size="sm" disabled={sibuk} onClick={onYa}>
          {labelYa}
        </Button>
        <Button variant="secondary" size="sm" disabled={sibuk} onClick={onBatal}>
          {TEKS.vps.pemeliharaan.batal}
        </Button>
      </div>
    </div>
  );
}

/** Baris data label + nilai (kartu VPS, Akses, pemeriksaan). */
export function Baris({ label, nilai, mono = false }: { label: string; nilai: ReactNode; mono?: boolean }) {
  return (
    <div className="flex items-start justify-between gap-3 py-1.5 border-b border-slate-100 dark:border-slate-800/80 last:border-b-0">
      <dt className="text-xs text-slate-500 dark:text-slate-400 shrink-0">{label}</dt>
      <dd
        className={`text-sm text-right text-slate-900 dark:text-slate-100 min-w-0 [overflow-wrap:anywhere] ${mono ? 'font-mono text-xs' : ''}`}
      >
        {nilai}
      </dd>
    </div>
  );
}
