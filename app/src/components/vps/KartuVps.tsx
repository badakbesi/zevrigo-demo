import { useState } from 'react';
import { RefreshCw, RotateCcw, Server } from 'lucide-react';
import { useVpsStore } from '../../lib/storeVps';
import { TEKS } from '../../lib/teks';
import { formatDurasi, formatMb, persenKuota } from '../../lib/vps';
import { Badge, Button, Card } from '../ui';
import { Baris, Konfirmasi } from './Formulir';

/** Keadaan VPS dari skrip bantu (`status`), dan restart VPS. */
export function KartuVps() {
  const status = useVpsStore((s) => s.status);
  const galatStatus = useVpsStore((s) => s.galatStatus);
  const sibuk = useVpsStore((s) => s.sibuk);
  const segarkanStatus = useVpsStore((s) => s.segarkanStatus);
  const kendali = useVpsStore((s) => s.kendali);
  const [konfirmasi, setKonfirmasi] = useState(false);

  const kuota = status ? persenKuota(status.kuotaCpu) : null;

  return (
    <Card className="space-y-3">
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <Server aria-hidden="true" className="w-4 h-4 text-slate-500" />
          <h2 className="text-base font-semibold">{TEKS.vps.status.judul}</h2>
        </div>
        {status && (
          <Badge tone={status.layanan === 'active' ? 'emerald' : 'rose'}>
            {TEKS.vps.status.layanan}: {status.layanan}
          </Badge>
        )}
      </div>
      {galatStatus && <p className="text-xs text-rose-700 dark:text-rose-300">{galatStatus}</p>}
      {status === null ? (
        <p className="text-sm text-slate-500 dark:text-slate-400">{TEKS.vps.memuat}</p>
      ) : (
        <dl>
          <Baris label={TEKS.vps.status.versi} nilai={status.versi || '-'} mono />
          <Baris label={TEKS.vps.status.cpu} nilai={`${status.cpu}`} />
          <Baris label={TEKS.vps.status.muatan} nilai={status.muatan1m} />
          <Baris
            label={TEKS.vps.status.ram}
            nilai={TEKS.vps.status.dari(formatMb(status.ramTersediaMb), formatMb(status.ramTotalMb))}
          />
          <Baris
            label={TEKS.vps.status.disk}
            nilai={TEKS.vps.status.dari(formatMb(status.diskBebasMb), formatMb(status.diskTotalMb))}
          />
          <Baris label={TEKS.vps.status.menyala} nilai={formatDurasi(status.uptimeDetik)} />
          <Baris
            label={TEKS.vps.status.nodePublik}
            nilai={status.nodePublik ? `${TEKS.vps.status.publik} (${status.nodePublik})` : TEKS.vps.status.rekanBiasa}
          />
          <Baris label={TEKS.vps.status.batasCpu} nilai={kuota === null ? TEKS.vps.status.tanpaBatas : `${kuota}%`} />
        </dl>
      )}
      {konfirmasi ? (
        <Konfirmasi
          id="judul-restart"
          judul={TEKS.vps.status.restartJudul}
          isi={TEKS.vps.status.restartIsi}
          bahaya
          sibuk={sibuk !== null}
          onYa={() => {
            setKonfirmasi(false);
            void kendali({ jenis: 'restart_vps' }, TEKS.vps.status.restartBerhasil);
          }}
          onBatal={() => setKonfirmasi(false)}
        />
      ) : (
        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="secondary" onClick={() => void segarkanStatus()}>
            <RefreshCw aria-hidden="true" className="w-3.5 h-3.5" />
            <span>{TEKS.vps.status.segarkan}</span>
          </Button>
          <Button size="sm" variant="secondary" disabled={sibuk !== null} onClick={() => setKonfirmasi(true)}>
            <RotateCcw aria-hidden="true" className="w-3.5 h-3.5" />
            <span>{TEKS.vps.status.restart}</span>
          </Button>
        </div>
      )}
    </Card>
  );
}
