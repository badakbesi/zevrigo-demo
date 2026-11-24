import { useState } from 'react';
import {
  CheckCircle2,
  ClipboardPaste,
  ExternalLink,
  Hammer,
  Square,
  XCircle,
} from 'lucide-react';
import type { StatusTambang } from '../lib/backend';
import {
  formatAngka,
  formatHitungMundur,
  formatWaktu,
  singkatIdBlok,
} from '../lib/format';
import { useAppStore } from '../lib/store';
import { TEKS } from '../lib/teks';
import { Button, Card, ErrorBox, StatusDot } from './ui';

const PILIHAN_JEDA: Array<{ nilai: 60 | 120 | 300; label: string; catatan?: string }> = [
  { nilai: 60, label: TEKS.tambang.opsiJeda.satuMenit },
  {
    nilai: 120,
    label: TEKS.tambang.opsiJeda.duaMenit,
    catatan: TEKS.tambang.opsiJeda.duaMenitRekomendasi,
  },
  { nilai: 300, label: TEKS.tambang.opsiJeda.limaMenit },
];

function nadaStatusTambang(status: StatusTambang): 'slate' | 'amber' | 'emerald' {
  switch (status) {
    case 'menambang':
      return 'emerald';
    case 'menunggu':
      return 'amber';
    case 'berhenti':
    default:
      return 'slate';
  }
}

export function KartuTambang() {
  const nodeStatus = useAppStore((s) => s.node.status);
  const tambang = useAppStore((s) => s.tambang);
  const alamatHadiah = useAppStore((s) => s.pengaturan.alamatHadiah);
  const jedaDetik = useAppStore((s) => s.pengaturan.jedaDetik);
  const hasilAlamat = useAppStore((s) => s.hasilAlamat);
  const sedangPeriksaAlamat = useAppStore((s) => s.sedangPeriksaAlamat);
  const sedangProsesTambang = useAppStore((s) => s.sedangProsesTambang);
  const ubahAlamatHadiah = useAppStore((s) => s.ubahAlamatHadiah);
  const ubahJedaDetik = useAppStore((s) => s.ubahJedaDetik);
  const mulaiTambang = useAppStore((s) => s.mulaiTambang);
  const berhentiTambang = useAppStore((s) => s.berhentiTambang);
  const bukaTautan = useAppStore((s) => s.bukaTautan);
  const salinTeks = useAppStore((s) => s.salinTeks);

  const [peringatanTempel, setPeringatanTempel] = useState<string | null>(null);

  const sedangJalan = tambang.status !== 'berhenti';
  const nodeSinkron = nodeStatus === 'sinkron';
  const alamatSah = hasilAlamat.sah && !sedangPeriksaAlamat;
  const bolehMulai = nodeSinkron && alamatSah && !sedangProsesTambang;

  let alasanNonaktif: string | null = null;
  if (!sedangJalan && !bolehMulai) {
    if (!nodeSinkron && !alamatSah) {
      alasanNonaktif = TEKS.tambang.alasanNonaktif.nodeDanAlamat;
    } else if (!nodeSinkron) {
      alasanNonaktif = TEKS.tambang.alasanNonaktif.nodeBelumSinkron;
    } else if (!alamatSah) {
      alasanNonaktif = TEKS.tambang.alasanNonaktif.alamatBelumSah;
    }
  }

  async function tanganiTempel(): Promise<void> {
    setPeringatanTempel(null);
    if (typeof navigator === 'undefined' || !navigator.clipboard?.readText) {
      setPeringatanTempel(TEKS.tambang.gagalTempel);
      return;
    }
    try {
      const isi = await navigator.clipboard.readText();
      await ubahAlamatHadiah(isi.trim());
    } catch {
      setPeringatanTempel(TEKS.tambang.gagalTempel);
    }
  }

  function teksValidasiAlamat(): {
    jenis: 'kosong' | 'sah' | 'galat' | 'memeriksa';
    teks: string;
  } {
    if (sedangPeriksaAlamat) {
      return { jenis: 'memeriksa', teks: TEKS.tambang.statusAlamat.memeriksa };
    }
    if (hasilAlamat.sah) {
      return { jenis: 'sah', teks: TEKS.tambang.statusAlamat.sah };
    }
    if (hasilAlamat.alasan === 'jaringan-lain') {
      return { jenis: 'galat', teks: TEKS.tambang.statusAlamat.jaringanLain };
    }
    if (hasilAlamat.alasan === 'format') {
      return { jenis: 'galat', teks: TEKS.tambang.statusAlamat.format };
    }
    return { jenis: 'kosong', teks: TEKS.tambang.statusAlamat.kosong };
  }

  const infoValidasi = teksValidasiAlamat();
  const teksStatus =
    tambang.status === 'menunggu'
      ? `${TEKS.tambang.status.menunggu} (${formatHitungMundur(tambang.sisaJedaDetik)})`
      : TEKS.tambang.status[tambang.status];

  return (
    <Card className="flex flex-col justify-between gap-3">
      <div className="space-y-3">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-base font-semibold text-slate-900 dark:text-slate-100">
            {TEKS.tambang.judul}
          </h2>
          <div
            aria-live="polite"
            aria-label={TEKS.tambang.labelStatus}
            className="inline-flex items-center gap-2 text-sm font-medium text-slate-700 dark:text-slate-200 tabular-nums"
          >
            <StatusDot tone={nadaStatusTambang(tambang.status)} />
            <span>{teksStatus}</span>
          </div>
        </div>

        <div className="space-y-1.5">
          <label
            htmlFor="alamat-hadiah"
            className="block text-xs font-medium text-slate-700 dark:text-slate-300"
          >
            {TEKS.tambang.labelAlamat}
          </label>
          <div className="flex items-center gap-2">
            <input
              id="alamat-hadiah"
              type="text"
              value={alamatHadiah}
              disabled={sedangJalan}
              onChange={(e) => {
                setPeringatanTempel(null);
                void ubahAlamatHadiah(e.target.value);
              }}
              placeholder={TEKS.tambang.placeholderAlamat}
              spellCheck={false}
              autoComplete="off"
              className="w-full min-w-0 rounded-lg border border-slate-300 dark:border-slate-700 bg-slate-50 dark:bg-slate-950 px-2.5 py-2 text-xs font-mono text-slate-900 dark:text-slate-100 placeholder:text-[11px] placeholder:text-slate-400 dark:placeholder:text-slate-500 disabled:opacity-60 focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-emerald-500"
            />
            <Button
              variant="secondary"
              size="sm"
              disabled={sedangJalan}
              onClick={() => void tanganiTempel()}
              className="py-2"
            >
              <ClipboardPaste className="w-3.5 h-3.5" aria-hidden="true" />
              <span>{TEKS.tambang.tombolTempel}</span>
            </Button>
          </div>

          <div
            aria-live="polite"
            className="flex items-start gap-1.5 text-xs leading-snug min-h-4"
          >
            {infoValidasi.jenis === 'sah' && (
              <>
                <CheckCircle2
                  aria-hidden="true"
                  className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400 mt-0.5 shrink-0"
                />
                <span className="text-emerald-700 dark:text-emerald-300 font-medium">
                  {infoValidasi.teks}
                </span>
              </>
            )}
            {infoValidasi.jenis === 'galat' && (
              <>
                <XCircle
                  aria-hidden="true"
                  className="w-3.5 h-3.5 text-rose-600 dark:text-rose-400 mt-0.5 shrink-0"
                />
                <span className="text-rose-700 dark:text-rose-300">
                  {infoValidasi.teks}
                </span>
              </>
            )}
            {(infoValidasi.jenis === 'kosong' || infoValidasi.jenis === 'memeriksa') && (
              <span className="text-slate-500 dark:text-slate-400">
                {infoValidasi.teks}
              </span>
            )}
          </div>
          {peringatanTempel && (
            <p className="text-xs text-amber-700 dark:text-amber-300">
              {peringatanTempel}
            </p>
          )}
        </div>

        <fieldset className="space-y-1.5" disabled={sedangJalan}>
          <legend className="text-xs font-medium text-slate-700 dark:text-slate-300">
            {TEKS.tambang.labelJeda}
          </legend>
          <div className="grid grid-cols-3 gap-2">
            {PILIHAN_JEDA.map((opsi) => {
              const terpilih = jedaDetik === opsi.nilai;
              return (
                <label
                  key={opsi.nilai}
                  className={`flex flex-col justify-center rounded-lg border px-2.5 py-2 text-xs transition-colors cursor-pointer ${
                    terpilih
                      ? 'border-emerald-500 bg-emerald-50/70 text-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-100 dark:border-emerald-600'
                      : 'border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950/60 text-slate-700 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-700'
                  } ${sedangJalan ? 'opacity-60 cursor-not-allowed' : ''}`}
                >
                  <div className="flex items-center gap-1.5">
                    <input
                      type="radio"
                      name="jeda-antar-blok"
                      value={opsi.nilai}
                      checked={terpilih}
                      onChange={() => void ubahJedaDetik(opsi.nilai)}
                      className="accent-emerald-600"
                    />
                    <span className="font-medium">{opsi.label}</span>
                  </div>
                  {opsi.catatan && (
                    <span className="mt-0.5 pl-4 text-[11px] leading-tight text-slate-500 dark:text-slate-400">
                      {opsi.catatan}
                    </span>
                  )}
                </label>
              );
            })}
          </div>
        </fieldset>

        <div className="space-y-1.5">
          {sedangJalan ? (
            <Button
              variant="danger"
              size="lg"
              className="w-full"
              disabled={sedangProsesTambang}
              onClick={() => void berhentiTambang()}
            >
              <Square className="w-4 h-4" aria-hidden="true" />
              <span>{TEKS.tambang.berhenti}</span>
            </Button>
          ) : (
            <Button
              variant="primary"
              size="lg"
              className="w-full"
              disabled={!bolehMulai}
              onClick={() => void mulaiTambang()}
            >
              <Hammer className="w-4 h-4" aria-hidden="true" />
              <span>{TEKS.tambang.mulai}</span>
            </Button>
          )}

          {alasanNonaktif && (
            <p className="text-xs text-amber-700 dark:text-amber-300 leading-snug">
              {alasanNonaktif}
            </p>
          )}
        </div>

        <div className="pt-2 border-t border-slate-100 dark:border-slate-800/80 space-y-2.5">
          <div className="flex items-center justify-between text-xs">
            <span className="text-slate-500 dark:text-slate-400">
              {TEKS.tambang.blokSesiIni}
            </span>
            <span className="font-mono font-semibold tabular-nums text-sm text-slate-900 dark:text-slate-100">
              {formatAngka(tambang.blokSesiIni)}
            </span>
          </div>

          <div className="space-y-1.5">
            <h3 className="text-xs font-medium text-slate-600 dark:text-slate-300">
              {TEKS.tambang.daftarBlokJudul}
            </h3>
            {tambang.blokTerakhir.length === 0 ? (
              <p className="text-xs text-slate-500 dark:text-slate-400">
                {TEKS.tambang.belumAdaBlok}
              </p>
            ) : (
              <ul className="divide-y divide-slate-100 dark:divide-slate-800 border border-slate-200 dark:border-slate-800 rounded-lg bg-slate-50/60 dark:bg-slate-950/50">
                {tambang.blokTerakhir.map((blok) => (
                  <li
                    key={blok.idBlok}
                    className="px-3 py-1.5 flex items-center justify-between gap-2 text-xs"
                  >
                    <div className="flex items-center gap-2.5 min-w-0 font-mono tabular-nums">
                      <span className="font-semibold text-slate-900 dark:text-slate-100">
                        {TEKS.tambang.tinggiBlokPrefix}
                        {formatAngka(blok.tinggi)}
                      </span>
                      <span className="text-slate-500 dark:text-slate-400">
                        {singkatIdBlok(blok.idBlok)}
                      </span>
                      <span className="text-slate-500 dark:text-slate-400">
                        {formatWaktu(blok.waktu)}
                      </span>
                    </div>
                    <button
                      type="button"
                      onClick={() =>
                        void bukaTautan(
                          `https://demo-explorer.zevrigo.online/block/?id=${blok.idBlok}`,
                        )
                      }
                      className="inline-flex items-center gap-1 text-emerald-700 hover:text-emerald-600 dark:text-emerald-400 dark:hover:text-emerald-300 font-medium whitespace-nowrap shrink-0 cursor-pointer focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-emerald-500 rounded"
                    >
                      <span>{TEKS.tambang.lihatDiExplorer}</span>
                      <ExternalLink className="w-3 h-3" aria-hidden="true" />
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>

        {tambang.pesanGalat && (
          <ErrorBox
            pesan={tambang.pesanGalat}
            detail={tambang.detailGalat}
            onSalinDetail={salinTeks}
          />
        )}
      </div>

      <p className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed pt-2 border-t border-slate-100 dark:border-slate-800/80">
        {TEKS.tambang.catatanHadiah}
      </p>
    </Card>
  );
}
