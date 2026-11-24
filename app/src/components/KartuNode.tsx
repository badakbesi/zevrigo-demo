import { useState } from 'react';
import { FolderOpen, Play, Square } from 'lucide-react';
import type { StatusNode } from '../lib/backend';
import { formatAngka } from '../lib/format';
import { useAppStore } from '../lib/store';
import { TEKS } from '../lib/teks';
import { Button, Card, ErrorBox, StatusDot } from './ui';

function nadaStatusNode(status: StatusNode): 'slate' | 'amber' | 'emerald' | 'rose' {
  switch (status) {
    case 'sinkron':
      return 'emerald';
    case 'menyala':
    case 'menyambung':
    case 'mengejar':
      return 'amber';
    case 'galat':
      return 'rose';
    case 'mati':
    default:
      return 'slate';
  }
}

export function KartuNode() {
  const node = useAppStore((s) => s.node);
  const tambangStatus = useAppStore((s) => s.tambang.status);
  const sedangProsesNode = useAppStore((s) => s.sedangProsesNode);
  const nyalakanNode = useAppStore((s) => s.nyalakanNode);
  const hentikanNode = useAppStore((s) => s.hentikanNode);
  const bukaFolderData = useAppStore((s) => s.bukaFolderData);
  const salinTeks = useAppStore((s) => s.salinTeks);

  const [konfirmasiHentikan, setKonfirmasiHentikan] = useState(false);

  const nodeAktif = node.status !== 'mati' && node.status !== 'galat';
  const nada = nadaStatusNode(node.status);
  const teksStatus = TEKS.node.status[node.status];

  const sedangMengejar =
    node.status === 'mengejar' &&
    node.tinggiLokal !== null &&
    node.tinggiJaringan !== null &&
    node.tinggiJaringan > 0;

  const persenMengejar = sedangMengejar
    ? Math.min(
        100,
        Math.max(
          0,
          Math.round(((node.tinggiLokal ?? 0) / (node.tinggiJaringan ?? 1)) * 100),
        ),
      )
    : 0;

  function tanganiTombolUtama(): void {
    if (!nodeAktif) {
      void nyalakanNode();
      return;
    }
    if (tambangStatus !== 'berhenti') {
      setKonfirmasiHentikan(true);
      return;
    }
    void hentikanNode();
  }

  async function konfirmasiYa(): Promise<void> {
    setKonfirmasiHentikan(false);
    await hentikanNode();
  }

  return (
    <Card className="flex flex-col justify-between gap-3">
      <div className="space-y-3">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-base font-semibold text-slate-900 dark:text-slate-100">
            {TEKS.node.judul}
          </h2>
          <div
            aria-live="polite"
            aria-label={TEKS.node.labelStatus}
            className="inline-flex items-center gap-2 text-sm font-medium text-slate-700 dark:text-slate-200"
          >
            <StatusDot tone={nada} />
            <span>{teksStatus}</span>
          </div>
        </div>

        <Button
          variant={nodeAktif ? 'secondary' : 'primary'}
          size="lg"
          className="w-full"
          disabled={sedangProsesNode}
          onClick={tanganiTombolUtama}
        >
          {nodeAktif ? (
            <Square className="w-4 h-4" aria-hidden="true" />
          ) : (
            <Play className="w-4 h-4" aria-hidden="true" />
          )}
          <span>
            {sedangProsesNode
              ? nodeAktif
                ? TEKS.node.sedangMenghentikan
                : TEKS.node.sedangMenyalakan
              : nodeAktif
                ? TEKS.node.hentikan
                : TEKS.node.nyalakan}
          </span>
        </Button>

        {konfirmasiHentikan && (
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="judul-konfirmasi-node"
            className="rounded-lg border border-amber-300 dark:border-amber-800 bg-amber-50 dark:bg-amber-950/50 p-3.5 space-y-3"
          >
            <div className="space-y-1">
              <p
                id="judul-konfirmasi-node"
                className="text-sm font-semibold text-amber-950 dark:text-amber-100"
              >
                {TEKS.node.konfirmasiHentikanJudul}
              </p>
              <p className="text-xs text-amber-900 dark:text-amber-200 leading-relaxed">
                {TEKS.node.konfirmasiHentikanIsi}
              </p>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <Button
                variant="danger"
                size="sm"
                onClick={() => void konfirmasiYa()}
              >
                {TEKS.node.konfirmasiYa}
              </Button>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => setKonfirmasiHentikan(false)}
              >
                {TEKS.node.konfirmasiBatal}
              </Button>
            </div>
          </div>
        )}

        {sedangMengejar && (
          <div className="space-y-1.5">
            <div className="flex items-center justify-between text-xs text-slate-600 dark:text-slate-300 tabular-nums">
              <span>
                {TEKS.node.kemajuanSinkron(
                  formatAngka(node.tinggiLokal),
                  formatAngka(node.tinggiJaringan),
                )}
              </span>
              <span className="font-mono">{persenMengejar}%</span>
            </div>
            <div
              role="progressbar"
              aria-valuemin={0}
              aria-valuemax={node.tinggiJaringan ?? 100}
              aria-valuenow={node.tinggiLokal ?? 0}
              aria-label={TEKS.node.status.mengejar}
              className="h-2 w-full rounded-full bg-slate-200 dark:bg-slate-800 overflow-hidden"
            >
              <div
                className="h-full bg-amber-500 transition-all duration-300"
                style={{ width: `${persenMengejar}%` }}
              />
            </div>
          </div>
        )}

        <dl className="grid grid-cols-3 gap-3 pt-1 border-t border-slate-100 dark:border-slate-800/80">
          <div className="pt-2">
            <dt className="text-xs text-slate-500 dark:text-slate-400">
              {TEKS.node.tinggiLokal}
            </dt>
            <dd className="mt-0.5 text-base font-semibold font-mono tabular-nums text-slate-900 dark:text-slate-100">
              {formatAngka(node.tinggiLokal)}
            </dd>
          </div>
          <div className="pt-2">
            <dt className="text-xs text-slate-500 dark:text-slate-400">
              {TEKS.node.tinggiJaringan}
            </dt>
            <dd className="mt-0.5 text-base font-semibold font-mono tabular-nums text-slate-900 dark:text-slate-100">
              {formatAngka(node.tinggiJaringan)}
            </dd>
          </div>
          <div className="pt-2">
            <dt className="text-xs text-slate-500 dark:text-slate-400">
              {TEKS.node.rekanTersambung}
            </dt>
            <dd className="mt-0.5 text-base font-semibold font-mono tabular-nums text-slate-900 dark:text-slate-100">
              {formatAngka(node.rekan)}
            </dd>
          </div>
        </dl>

        <div className="pt-2 border-t border-slate-100 dark:border-slate-800/80 space-y-1.5">
          <span className="block text-xs text-slate-500 dark:text-slate-400">
            {TEKS.node.folderData}
          </span>
          <div className="flex items-center justify-between gap-2">
            <span
              title={node.folderData}
              className="font-mono text-xs text-slate-700 dark:text-slate-300 truncate min-w-0"
            >
              {node.folderData || '-'}
            </span>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => void bukaFolderData()}
            >
              <FolderOpen className="w-3.5 h-3.5" aria-hidden="true" />
              <span>{TEKS.node.bukaFolder}</span>
            </Button>
          </div>
        </div>

        {node.pesanGalat && (
          <ErrorBox
            pesan={node.pesanGalat}
            detail={node.detailGalat}
            onSalinDetail={salinTeks}
          />
        )}
      </div>

      <p className="text-xs text-slate-500 dark:text-slate-400 leading-relaxed pt-2 border-t border-slate-100 dark:border-slate-800/80">
        {TEKS.node.bantuan}
      </p>
    </Card>
  );
}
