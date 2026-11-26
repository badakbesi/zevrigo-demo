import { useState } from 'react';
import { Wrench } from 'lucide-react';
import { useVpsStore } from '../../lib/storeVps';
import { TEKS } from '../../lib/teks';
import { persenKuota } from '../../lib/vps';
import { Button, Card } from '../ui';
import { Isian, Konfirmasi } from './Formulir';

const PILIHAN_CPU = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];

type JenisKonfirmasi = 'perbarui' | 'copot' | null;

/** Perbarui node, batas CPU, node publik, token pengendali, copot. */
export function KartuPemeliharaan() {
  const info = useVpsStore((s) => s.info);
  const status = useVpsStore((s) => s.status);
  const sibuk = useVpsStore((s) => s.sibuk);
  const kendali = useVpsStore((s) => s.kendali);
  const gantiToken = useVpsStore((s) => s.gantiToken);
  const copot = useVpsStore((s) => s.copot);
  const [konfirmasi, setKonfirmasi] = useState<JenisKonfirmasi>(null);
  const [hapusData, setHapusData] = useState(false);
  const [cpu, setCpu] = useState<number | null>(null);
  const [ip, setIp] = useState('');
  const [portP2p, setPortP2p] = useState('28800');

  const cpuSekarang = (status ? persenKuota(status.kuotaCpu) : null) ?? 50;
  const cpuPilih = cpu ?? cpuSekarang;
  const portAngka = Number(portP2p);
  const portSah = Number.isInteger(portAngka) && portAngka >= 1024 && portAngka <= 65535;
  const nonaktif = sibuk !== null;
  const sha = info?.paketSha256.slice(0, 12) ?? '';

  return (
    <Card className="space-y-4">
      <div className="flex items-center gap-2">
        <Wrench aria-hidden="true" className="w-4 h-4 text-slate-500" />
        <h2 className="text-base font-semibold">{TEKS.vps.pemeliharaan.judul}</h2>
      </div>

      <section className="space-y-2">
        <h3 className="text-sm font-medium">{TEKS.vps.pemeliharaan.perbarui}</h3>
        <p className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed">
          {TEKS.vps.pemeliharaan.perbaruiIsi(sha)}
        </p>
        {konfirmasi === 'perbarui' ? (
          <Konfirmasi
            id="judul-perbarui"
            judul={TEKS.vps.pemeliharaan.perbaruiJudul}
            isi={TEKS.vps.pemeliharaan.perbaruiIsi(sha)}
            sibuk={nonaktif}
            onYa={() => {
              setKonfirmasi(null);
              void kendali({ jenis: 'perbarui' }, TEKS.vps.pemeliharaan.perbaruiBerhasil);
            }}
            onBatal={() => setKonfirmasi(null)}
          />
        ) : (
          <Button size="sm" variant="secondary" disabled={nonaktif} onClick={() => setKonfirmasi('perbarui')}>
            {TEKS.vps.pemeliharaan.perbarui}
          </Button>
        )}
      </section>

      <section className="space-y-2 pt-3 border-t border-slate-100 dark:border-slate-800/80">
        <h3 className="text-sm font-medium">{TEKS.vps.pemeliharaan.batasCpu}</h3>
        <p className="text-xs text-slate-500 dark:text-slate-400">{TEKS.vps.pemeliharaan.batasCpuIsi}</p>
        <div className="flex items-center gap-2">
          <label htmlFor="batas-cpu" className="sr-only">
            {TEKS.vps.pemeliharaan.batasCpu}
          </label>
          <select
            id="batas-cpu"
            value={cpuPilih}
            onChange={(e) => setCpu(Number(e.target.value))}
            className="rounded-lg border border-slate-300 dark:border-slate-700 bg-white dark:bg-slate-950 px-3 py-1.5 text-sm"
          >
            {PILIHAN_CPU.map((n) => (
              <option key={n} value={n}>
                {n}%
              </option>
            ))}
          </select>
          <Button
            size="sm"
            variant="secondary"
            disabled={nonaktif || cpuPilih === cpuSekarang}
            onClick={() =>
              void kendali(
                { jenis: 'batas_cpu', persen: cpuPilih },
                TEKS.vps.pemeliharaan.batasCpuBerhasil(cpuPilih),
              ).then(() => setCpu(null))
            }
          >
            {TEKS.vps.pemeliharaan.simpan}
          </Button>
        </div>
      </section>

      <section className="space-y-2 pt-3 border-t border-slate-100 dark:border-slate-800/80">
        <h3 className="text-sm font-medium">{TEKS.vps.pemeliharaan.nodePublik}</h3>
        <p className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed">{TEKS.vps.pemeliharaan.nodePublikIsi}</p>
        <div className="grid grid-cols-3 gap-2">
          <Isian
            id="ip-publik"
            className="col-span-2"
            label={TEKS.vps.pemeliharaan.labelIp}
            nilai={ip}
            ubah={setIp}
            inputMode="decimal"
            autoCapitalize="off"
            autoCorrect="off"
            spellCheck={false}
            placeholder={info?.host ?? ''}
          />
          <Isian
            id="port-p2p"
            label={TEKS.vps.pemeliharaan.labelPortP2p}
            nilai={portP2p}
            ubah={setPortP2p}
            inputMode="numeric"
          />
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant="secondary"
            disabled={nonaktif || ip.trim() === '' || !portSah}
            onClick={() =>
              void kendali(
                { jenis: 'node_publik_nyala', ip: ip.trim(), port: portAngka },
                TEKS.vps.pemeliharaan.nodePublikBerhasil,
              )
            }
          >
            {TEKS.vps.pemeliharaan.nyalakan}
          </Button>
          <Button
            size="sm"
            variant="secondary"
            disabled={nonaktif || !status?.nodePublik}
            onClick={() => void kendali({ jenis: 'node_publik_mati' }, TEKS.vps.pemeliharaan.nodePublikBerhasil)}
          >
            {TEKS.vps.pemeliharaan.matikan}
          </Button>
        </div>
      </section>

      <section className="space-y-2 pt-3 border-t border-slate-100 dark:border-slate-800/80">
        <h3 className="text-sm font-medium">{TEKS.vps.pemeliharaan.gantiToken}</h3>
        <p className="text-xs text-slate-500 dark:text-slate-400">{TEKS.vps.pemeliharaan.gantiTokenIsi}</p>
        <Button size="sm" variant="secondary" disabled={nonaktif} onClick={() => void gantiToken()}>
          {TEKS.vps.pemeliharaan.gantiToken}
        </Button>
      </section>

      <section className="space-y-2 pt-3 border-t border-slate-100 dark:border-slate-800/80">
        {konfirmasi === 'copot' ? (
          <Konfirmasi
            id="judul-copot"
            judul={TEKS.vps.pemeliharaan.copotJudul}
            isi={TEKS.vps.pemeliharaan.copotIsi}
            bahaya
            sibuk={nonaktif}
            onYa={() => {
              setKonfirmasi(null);
              void copot(hapusData);
            }}
            onBatal={() => setKonfirmasi(null)}
          >
            <label className="flex items-center gap-2 text-xs text-amber-950 dark:text-amber-100">
              <input type="checkbox" checked={hapusData} onChange={(e) => setHapusData(e.target.checked)} />
              <span>{TEKS.vps.pemeliharaan.copotHapusData}</span>
            </label>
          </Konfirmasi>
        ) : (
          <Button size="sm" variant="danger" disabled={nonaktif} onClick={() => setKonfirmasi('copot')}>
            {TEKS.vps.pemeliharaan.copot}
          </Button>
        )}
      </section>
    </Card>
  );
}
