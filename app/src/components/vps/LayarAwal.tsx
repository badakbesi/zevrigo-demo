import { useState } from 'react';
import { KeyRound, Server, Upload } from 'lucide-react';
import { useVpsStore } from '../../lib/storeVps';
import { TEKS } from '../../lib/teks';
import { Button, Card } from '../ui';
import { AreaTeks, GalatVpsBox, Isian, Sedang } from './Formulir';

/** Tahap baru: buat kata sandi aplikasi, atau impor data VPS dari HP lama. */
export function LayarAwal() {
  const info = useVpsStore((s) => s.info);
  const sibuk = useVpsStore((s) => s.sibuk);
  const galat = useVpsStore((s) => s.galat);
  const buatSandi = useVpsStore((s) => s.buatSandi);
  const impor = useVpsStore((s) => s.impor);
  const bersihkanGalat = useVpsStore((s) => s.bersihkanGalat);
  const [sandi, setSandi] = useState('');
  const [ulang, setUlang] = useState('');
  const [modeImpor, setModeImpor] = useState(false);
  const [teksImpor, setTeksImpor] = useState('');
  const [sandiImpor, setSandiImpor] = useState('');

  const min = info?.sandiMin ?? 8;
  const panjang = [...sandi].length;
  const bisaLanjut = panjang >= min && ulang.length > 0 && sibuk === null;

  function gantiMode(): void {
    bersihkanGalat();
    setModeImpor((v) => !v);
  }

  return (
    <div className="space-y-3">
      <Card className="space-y-2">
        <div className="flex items-center gap-2">
          <Server aria-hidden="true" className="w-5 h-5 text-emerald-600 dark:text-emerald-400" />
          <h2 className="text-base font-semibold">{TEKS.vps.awal.judul}</h2>
        </div>
        <p className="text-sm text-slate-600 dark:text-slate-300 leading-relaxed">{TEKS.vps.awal.isi}</p>
        <p className="text-xs text-slate-500 dark:text-slate-400">{TEKS.vps.awal.syarat}</p>
      </Card>

      {!modeImpor ? (
        <Card className="space-y-3">
          <div className="flex items-center gap-2">
            <KeyRound aria-hidden="true" className="w-4 h-4 text-slate-500" />
            <h2 className="text-base font-semibold">{TEKS.vps.awal.buatJudul}</h2>
          </div>
          <p className="text-sm text-slate-600 dark:text-slate-300 leading-relaxed">{TEKS.vps.awal.buatIsi(min)}</p>
          <form
            className="space-y-3"
            onSubmit={(e) => {
              e.preventDefault();
              if (bisaLanjut) {
                void buatSandi(sandi, ulang);
              }
            }}
          >
            <Isian
              id="sandi-baru"
              label={`${TEKS.vps.awal.labelSandi} (${panjang}/${min})`}
              type="password"
              autoComplete="new-password"
              nilai={sandi}
              ubah={setSandi}
            />
            <Isian
              id="sandi-ulang"
              label={TEKS.vps.awal.labelUlang}
              type="password"
              autoComplete="new-password"
              nilai={ulang}
              ubah={setUlang}
            />
            <p className="rounded-lg border border-amber-200 dark:border-amber-900/60 bg-amber-50 dark:bg-amber-950/40 p-3 text-xs text-amber-900 dark:text-amber-200 leading-relaxed">
              {TEKS.vps.awal.peringatanLupa}
            </p>
            <GalatVpsBox galat={galat} />
            <Button type="submit" variant="primary" size="lg" className="w-full" disabled={!bisaLanjut}>
              {TEKS.vps.awal.lanjut}
            </Button>
          </form>
        </Card>
      ) : (
        <Card className="space-y-3">
          <div className="flex items-center gap-2">
            <Upload aria-hidden="true" className="w-4 h-4 text-slate-500" />
            <h2 className="text-base font-semibold">{TEKS.vps.awal.imporJudul}</h2>
          </div>
          <p className="text-sm text-slate-600 dark:text-slate-300 leading-relaxed">{TEKS.vps.awal.imporIsi}</p>
          <AreaTeks id="teks-impor" label={TEKS.vps.awal.labelImpor} nilai={teksImpor} ubah={setTeksImpor} />
          <Isian
            id="sandi-impor"
            label={TEKS.vps.awal.labelSandi}
            type="password"
            autoComplete="current-password"
            nilai={sandiImpor}
            ubah={setSandiImpor}
          />
          <GalatVpsBox galat={galat} />
          {sibuk === 'impor' && <Sedang teks={TEKS.vps.memuat} />}
          <Button
            variant="primary"
            size="lg"
            className="w-full"
            disabled={sibuk !== null || teksImpor.trim() === '' || sandiImpor === ''}
            onClick={() => void impor(teksImpor, sandiImpor)}
          >
            {TEKS.vps.awal.impor}
          </Button>
        </Card>
      )}

      <div className="text-center">
        <Button variant="ghost" size="sm" onClick={gantiMode} disabled={sibuk !== null}>
          {modeImpor ? TEKS.vps.awal.buatJudul : TEKS.vps.awal.bukaImpor}
        </Button>
      </div>
    </div>
  );
}
