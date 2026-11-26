import { AlertTriangle, Download, ShieldCheck } from 'lucide-react';
import { formatMb } from '../../lib/vps';
import { useVpsStore } from '../../lib/storeVps';
import { TEKS } from '../../lib/teks';
import { Button, Card } from '../ui';
import { Baris, GalatVpsBox, Sedang } from './Formulir';

/** Tahap pasang: sidik kunci VPS, hasil pemeriksaan, lalu pasang (atau ambil alih node yang sudah ada). */
export function LayarPasang() {
  const info = useVpsStore((s) => s.info);
  const sibuk = useVpsStore((s) => s.sibuk);
  const galat = useVpsStore((s) => s.galat);
  const pasang = useVpsStore((s) => s.pasang);
  const batal = useVpsStore((s) => s.batal);
  const p = info?.pemeriksaan ?? null;
  const masalah = info?.masalah ?? [];

  return (
    <Card className="space-y-3">
      <div className="flex items-center gap-2">
        <Download aria-hidden="true" className="w-4 h-4 text-slate-500" />
        <h2 className="text-base font-semibold">{TEKS.vps.pasang.judul}</h2>
      </div>
      <p className="text-sm font-mono text-slate-700 dark:text-slate-200 [overflow-wrap:anywhere]">
        {info?.pengguna}@{info?.host}:{info?.port}
      </p>

      <div className="rounded-lg border border-slate-200 dark:border-slate-800 p-3 space-y-1.5">
        <div className="flex items-center gap-2 text-xs font-medium text-slate-600 dark:text-slate-300">
          <ShieldCheck aria-hidden="true" className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
          <span>{TEKS.vps.pasang.sidik}</span>
        </div>
        <p className="font-mono text-xs text-slate-900 dark:text-slate-100 [overflow-wrap:anywhere]">{info?.sidik}</p>
        <p className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed">{TEKS.vps.pasang.sidikIsi}</p>
      </div>

      {p !== null && (
        <dl>
          <Baris label={TEKS.vps.pasang.sistem} nilai={p.namaOs || `${p.os} ${p.versiOs}`} />
          <Baris label={TEKS.vps.pasang.arsitektur} nilai={p.arsitektur} />
          <Baris label={TEKS.vps.pasang.ram} nilai={formatMb(p.ramMb)} />
          <Baris label={TEKS.vps.pasang.disk} nilai={formatMb(p.diskBebasMb)} />
          <Baris label={TEKS.vps.pasang.akses} nilai={p.root ? TEKS.vps.pasang.aksesRoot : TEKS.vps.pasang.aksesSudo} />
          <Baris label={TEKS.vps.pasang.paket} nilai={`${info?.paketSha256.slice(0, 16)}...`} mono />
        </dl>
      )}

      {masalah.length > 0 && (
        <div role="alert" className="rounded-lg border border-rose-200 dark:border-rose-900/70 bg-rose-50 dark:bg-rose-950/40 p-3 space-y-1.5 text-sm text-rose-900 dark:text-rose-100">
          <p className="font-medium">{TEKS.vps.pasang.masalahJudul}</p>
          <ul className="list-disc pl-5 space-y-0.5 text-xs">
            {masalah.map((m) => (
              <li key={m}>{m}</li>
            ))}
          </ul>
        </div>
      )}

      {p?.terpasang && (
        <p className="flex items-start gap-2 rounded-lg border border-amber-200 dark:border-amber-900/60 bg-amber-50 dark:bg-amber-950/40 p-3 text-xs text-amber-900 dark:text-amber-200 leading-relaxed">
          <AlertTriangle aria-hidden="true" className="w-3.5 h-3.5 mt-0.5 shrink-0" />
          <span>{TEKS.vps.pasang.sudahAda}</span>
        </p>
      )}

      <GalatVpsBox galat={galat} />
      {sibuk === 'pasang' && <Sedang teks={TEKS.vps.pasang.sedang} />}
      <div className="flex flex-wrap gap-2">
        <Button
          variant="primary"
          size="lg"
          className="flex-1"
          disabled={sibuk !== null || masalah.length > 0}
          onClick={() => void pasang()}
        >
          {p?.terpasang ? TEKS.vps.pasang.tombolAmbil : TEKS.vps.pasang.tombolPasang}
        </Button>
        <Button variant="secondary" size="lg" disabled={sibuk !== null} onClick={() => void batal()}>
          {TEKS.vps.pasang.batal}
        </Button>
      </div>
    </Card>
  );
}
