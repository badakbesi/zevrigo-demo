import { useState } from 'react';
import { Lock } from 'lucide-react';
import { useVpsStore } from '../../lib/storeVps';
import { TEKS } from '../../lib/teks';
import { Button, Card } from '../ui';
import { GalatVpsBox, Isian, Konfirmasi, Sedang } from './Formulir';

/** Tahap terkunci: kata sandi aplikasi. */
export function LayarKunci() {
  const sibuk = useVpsStore((s) => s.sibuk);
  const galat = useVpsStore((s) => s.galat);
  const buka = useVpsStore((s) => s.buka);
  const lupakan = useVpsStore((s) => s.lupakan);
  const [sandi, setSandi] = useState('');
  const [lupa, setLupa] = useState(false);

  return (
    <Card className="space-y-3 max-w-md mx-auto">
      <div className="flex items-center gap-2">
        <Lock aria-hidden="true" className="w-4 h-4 text-slate-500" />
        <h2 className="text-base font-semibold">{TEKS.vps.kunciLayar.judul}</h2>
      </div>
      <p className="text-sm text-slate-600 dark:text-slate-300 leading-relaxed">{TEKS.vps.kunciLayar.isi}</p>
      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          if (sandi !== '' && sibuk === null) {
            void buka(sandi).then(() => setSandi(''));
          }
        }}
      >
        <Isian
          id="sandi-buka"
          label={TEKS.vps.kunciLayar.labelSandi}
          type="password"
          autoComplete="current-password"
          autoFocus
          nilai={sandi}
          ubah={setSandi}
        />
        <GalatVpsBox galat={galat} />
        {sibuk === 'buka' && <Sedang teks={TEKS.vps.kunciLayar.sedang} />}
        <Button type="submit" variant="primary" size="lg" className="w-full" disabled={sandi === '' || sibuk !== null}>
          {TEKS.vps.kunciLayar.buka}
        </Button>
      </form>
      {!lupa ? (
        <div className="text-center">
          <Button variant="ghost" size="sm" onClick={() => setLupa(true)}>
            {TEKS.vps.kunciLayar.lupa}
          </Button>
        </div>
      ) : (
        <Konfirmasi
          id="judul-lupa"
          judul={TEKS.vps.kunciLayar.lupa}
          isi={TEKS.vps.kunciLayar.lupaIsi}
          labelYa={TEKS.vps.kunciLayar.hapus}
          bahaya
          sibuk={sibuk !== null}
          onYa={() => void lupakan()}
          onBatal={() => setLupa(false)}
        />
      )}
    </Card>
  );
}
