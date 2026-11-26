import { ShieldAlert } from 'lucide-react';
import { useVpsStore } from '../../lib/storeVps';
import { TEKS } from '../../lib/teks';
import { Button, Card } from '../ui';
import { GalatVpsBox, Sedang } from './Formulir';

/** Tahap amankan: penjelasan konsol web penyedia VPS, lalu login password SSH dimatikan. */
export function LayarAmankan() {
  const sibuk = useVpsStore((s) => s.sibuk);
  const galat = useVpsStore((s) => s.galat);
  const amankan = useVpsStore((s) => s.amankan);
  const lewatiAmankan = useVpsStore((s) => s.lewatiAmankan);

  return (
    <Card className="space-y-3">
      <div className="flex items-center gap-2">
        <ShieldAlert aria-hidden="true" className="w-4 h-4 text-amber-600 dark:text-amber-400" />
        <h2 className="text-base font-semibold">{TEKS.vps.amankan.judul}</h2>
      </div>
      <p className="text-sm text-slate-600 dark:text-slate-300 leading-relaxed">{TEKS.vps.amankan.isi}</p>
      <p className="rounded-lg border border-amber-200 dark:border-amber-900/60 bg-amber-50 dark:bg-amber-950/40 p-3 text-xs text-amber-900 dark:text-amber-200 leading-relaxed">
        {TEKS.vps.amankan.darurat}
      </p>
      <GalatVpsBox galat={galat} />
      {sibuk === 'amankan' && <Sedang teks={TEKS.vps.amankan.sedang} />}
      <Button variant="primary" size="lg" className="w-full" disabled={sibuk !== null} onClick={() => void amankan()}>
        {TEKS.vps.amankan.tombol}
      </Button>
      {galat !== null && (
        <div className="space-y-1 text-center">
          <Button variant="ghost" size="sm" disabled={sibuk !== null} onClick={() => void lewatiAmankan()}>
            {TEKS.vps.amankan.lewati}
          </Button>
          <p className="text-xs text-slate-500 dark:text-slate-400">{TEKS.vps.amankan.lewatiIsi}</p>
        </div>
      )}
    </Card>
  );
}
