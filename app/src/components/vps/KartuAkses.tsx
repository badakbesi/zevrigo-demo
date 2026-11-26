import { useState } from 'react';
import { Check, Copy, KeyRound } from 'lucide-react';
import { useAppStore } from '../../lib/store';
import { useVpsStore } from '../../lib/storeVps';
import { TEKS } from '../../lib/teks';
import { Button, Card } from '../ui';
import { AreaTeks, Baris, Konfirmasi } from './Formulir';

type JenisKonfirmasi = 'cabut' | 'hapus' | null;

/** VPS dan sidik kunci, login password, ekspor data VPS (pindah HP), cabut akses, hapus data dari HP. */
export function KartuAkses() {
  const info = useVpsStore((s) => s.info);
  const status = useVpsStore((s) => s.status);
  const sibuk = useVpsStore((s) => s.sibuk);
  const teksEkspor = useVpsStore((s) => s.teksEkspor);
  const amankan = useVpsStore((s) => s.amankan);
  const ekspor = useVpsStore((s) => s.ekspor);
  const tutupEkspor = useVpsStore((s) => s.tutupEkspor);
  const cabutAkses = useVpsStore((s) => s.cabutAkses);
  const lupakan = useVpsStore((s) => s.lupakan);
  const salinTeks = useAppStore((s) => s.salinTeks);
  const [konfirmasi, setKonfirmasi] = useState<JenisKonfirmasi>(null);
  const [tersalin, setTersalin] = useState(false);

  const loginMati = status ? status.loginPassword === 'no' : info?.amankanSelesai === true;
  const nonaktif = sibuk !== null;

  async function salin(teks: string): Promise<void> {
    await salinTeks(teks);
    setTersalin(true);
    setTimeout(() => setTersalin(false), 1800);
  }

  return (
    <Card className="space-y-3">
      <div className="flex items-center gap-2">
        <KeyRound aria-hidden="true" className="w-4 h-4 text-slate-500" />
        <h2 className="text-base font-semibold">{TEKS.vps.akses.judul}</h2>
      </div>
      <dl>
        <Baris label={TEKS.vps.akses.vps} nilai={`${info?.pengguna ?? ''}@${info?.host ?? ''}:${info?.port ?? ''}`} mono />
        <Baris label={TEKS.vps.akses.sidik} nilai={info?.sidik ?? '-'} mono />
        <Baris
          label={TEKS.vps.akses.loginPassword}
          nilai={loginMati ? TEKS.vps.akses.loginMati : TEKS.vps.akses.loginNyala}
        />
      </dl>
      {!loginMati && (
        <Button size="sm" variant="primary" disabled={nonaktif} onClick={() => void amankan()}>
          {TEKS.vps.akses.matikanLogin}
        </Button>
      )}

      <section className="space-y-2 pt-3 border-t border-slate-100 dark:border-slate-800/80">
        {teksEkspor === null ? (
          <Button size="sm" variant="secondary" disabled={nonaktif} onClick={() => void ekspor()}>
            {TEKS.vps.akses.ekspor}
          </Button>
        ) : (
          <div className="space-y-2">
            <p className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed">{TEKS.vps.akses.eksporIsi}</p>
            <AreaTeks id="teks-ekspor" label={TEKS.vps.akses.ekspor} nilai={teksEkspor} bacaSaja />
            <div className="flex gap-2">
              <Button size="sm" variant="secondary" onClick={() => void salin(teksEkspor)}>
                {tersalin ? (
                  <Check aria-hidden="true" className="w-3.5 h-3.5" />
                ) : (
                  <Copy aria-hidden="true" className="w-3.5 h-3.5" />
                )}
                <span>{tersalin ? TEKS.vps.akses.tersalin : TEKS.vps.akses.salin}</span>
              </Button>
              <Button size="sm" variant="ghost" onClick={tutupEkspor}>
                {TEKS.vps.akses.tutup}
              </Button>
            </div>
          </div>
        )}
      </section>

      <section className="space-y-2 pt-3 border-t border-slate-100 dark:border-slate-800/80">
        {konfirmasi === 'cabut' && (
          <Konfirmasi
            id="judul-cabut"
            judul={TEKS.vps.akses.cabutJudul}
            isi={TEKS.vps.akses.cabutIsi}
            bahaya
            sibuk={nonaktif}
            onYa={() => {
              setKonfirmasi(null);
              void cabutAkses();
            }}
            onBatal={() => setKonfirmasi(null)}
          />
        )}
        {konfirmasi === 'hapus' && (
          <Konfirmasi
            id="judul-hapus"
            judul={TEKS.vps.akses.hapusJudul}
            isi={TEKS.vps.akses.hapusIsi}
            bahaya
            sibuk={nonaktif}
            onYa={() => {
              setKonfirmasi(null);
              void lupakan();
            }}
            onBatal={() => setKonfirmasi(null)}
          />
        )}
        {konfirmasi === null && (
          <div className="flex flex-wrap gap-2">
            <Button size="sm" variant="danger" disabled={nonaktif} onClick={() => setKonfirmasi('cabut')}>
              {TEKS.vps.akses.cabut}
            </Button>
            <Button size="sm" variant="secondary" disabled={nonaktif} onClick={() => setKonfirmasi('hapus')}>
              {TEKS.vps.akses.hapus}
            </Button>
          </div>
        )}
      </section>
    </Card>
  );
}
