import { Wallet } from 'lucide-react';
import { periksaAlamatAwal } from '../../lib/alamat';
import { formatAngka } from '../../lib/format';
import { useAppStore } from '../../lib/store';
import { useVpsStore } from '../../lib/storeVps';
import { TEKS } from '../../lib/teks';
import { formatZvrg } from '../../lib/vps';
import { Card } from '../ui';

/** Saldo alamat hadiah dari node di VPS (metode pengendali `saldo`), tanpa kunci wallet. */
export function KartuSaldo() {
  const saldo = useVpsStore((s) => s.saldo);
  const alamat = useAppStore((s) => s.pengaturan.alamatHadiah);
  const sah = periksaAlamatAwal(alamat).sah;

  return (
    <Card className="space-y-3">
      <div className="flex items-center gap-2">
        <Wallet aria-hidden="true" className="w-4 h-4 text-slate-500" />
        <h2 className="text-base font-semibold">{TEKS.vps.saldo.judul}</h2>
      </div>
      {!sah ? (
        <p className="text-sm text-slate-500 dark:text-slate-400">{TEKS.vps.saldo.isiAlamat}</p>
      ) : saldo === null ? (
        <p className="text-sm text-slate-500 dark:text-slate-400">{TEKS.vps.memuat}</p>
      ) : (
        <dl className="grid grid-cols-2 gap-3">
          <div>
            <dt className="text-xs text-slate-500 dark:text-slate-400">{TEKS.vps.saldo.matang}</dt>
            <dd className="mt-0.5 text-base font-semibold font-mono tabular-nums">{formatZvrg(saldo.matang)} ZVRG</dd>
          </div>
          <div>
            <dt className="text-xs text-slate-500 dark:text-slate-400">{TEKS.vps.saldo.belumMatang}</dt>
            <dd className="mt-0.5 text-base font-semibold font-mono tabular-nums">
              {formatZvrg(saldo.belumMatang)} ZVRG
            </dd>
            <dd className="text-[11px] text-slate-500 dark:text-slate-400">{TEKS.vps.saldo.belumMatangIsi}</dd>
          </div>
          <div>
            <dt className="text-xs text-slate-500 dark:text-slate-400">{TEKS.vps.saldo.output}</dt>
            <dd className="mt-0.5 text-sm font-mono tabular-nums">{formatAngka(saldo.jumlahOutput)}</dd>
          </div>
          <div>
            <dt className="text-xs text-slate-500 dark:text-slate-400">{TEKS.vps.saldo.tinggi}</dt>
            <dd className="mt-0.5 text-sm font-mono tabular-nums">{formatAngka(saldo.tinggi)}</dd>
          </div>
        </dl>
      )}
      <p className="text-xs text-slate-500 dark:text-slate-400 pt-2 border-t border-slate-100 dark:border-slate-800/80">
        {TEKS.vps.saldo.catatan}
      </p>
    </Card>
  );
}
