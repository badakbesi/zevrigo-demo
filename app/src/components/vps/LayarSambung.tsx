import { useState } from 'react';
import { Plug } from 'lucide-react';
import { useVpsStore } from '../../lib/storeVps';
import { TEKS } from '../../lib/teks';
import { Button, Card } from '../ui';
import { GalatVpsBox, Isian, Sedang } from './Formulir';

/** Tahap sambung: data SSH dari penyedia VPS. */
export function LayarSambung() {
  const info = useVpsStore((s) => s.info);
  const sibuk = useVpsStore((s) => s.sibuk);
  const galat = useVpsStore((s) => s.galat);
  const sambung = useVpsStore((s) => s.sambung);
  const batal = useVpsStore((s) => s.batal);
  const [host, setHost] = useState(info?.host ?? '');
  const [port, setPort] = useState(String(info?.port ?? 22));
  const [pengguna, setPengguna] = useState(info?.pengguna ?? 'root');
  const [sandi, setSandi] = useState('');

  const portAngka = Number(port);
  const portSah = Number.isInteger(portAngka) && portAngka > 0 && portAngka <= 65535;
  const bisa = host.trim() !== '' && pengguna.trim() !== '' && sandi !== '' && portSah && sibuk === null;

  return (
    <Card className="space-y-3">
      <div className="flex items-center gap-2">
        <Plug aria-hidden="true" className="w-4 h-4 text-slate-500" />
        <h2 className="text-base font-semibold">{TEKS.vps.sambung.judul}</h2>
      </div>
      <p className="text-sm text-slate-600 dark:text-slate-300 leading-relaxed">{TEKS.vps.sambung.isi}</p>
      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          if (bisa) {
            void sambung(host, portAngka, pengguna, sandi);
          }
        }}
      >
        <div className="grid grid-cols-3 gap-3">
          <Isian
            id="vps-host"
            className="col-span-2"
            label={TEKS.vps.sambung.labelHost}
            nilai={host}
            ubah={setHost}
            autoCapitalize="off"
            autoCorrect="off"
            spellCheck={false}
            inputMode="url"
            placeholder="203.0.113.7"
          />
          <Isian
            id="vps-port"
            label={TEKS.vps.sambung.labelPort}
            nilai={port}
            ubah={setPort}
            inputMode="numeric"
          />
        </div>
        <Isian
          id="vps-pengguna"
          label={TEKS.vps.sambung.labelPengguna}
          nilai={pengguna}
          ubah={setPengguna}
          autoCapitalize="off"
          autoCorrect="off"
          spellCheck={false}
          autoComplete="username"
        />
        <Isian
          id="vps-sandi"
          label={TEKS.vps.sambung.labelSandi}
          type="password"
          autoComplete="current-password"
          nilai={sandi}
          ubah={setSandi}
        />
        <GalatVpsBox galat={galat} />
        {sibuk === 'sambung' && <Sedang teks={TEKS.vps.sambung.sedang} />}
        <div className="flex flex-wrap gap-2">
          <Button type="submit" variant="primary" size="lg" className="flex-1" disabled={!bisa}>
            {TEKS.vps.sambung.tombol}
          </Button>
          <Button variant="secondary" size="lg" disabled={sibuk !== null} onClick={() => void batal()}>
            {TEKS.vps.sambung.batal}
          </Button>
        </div>
      </form>
    </Card>
  );
}
