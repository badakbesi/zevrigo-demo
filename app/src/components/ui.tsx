import type { ButtonHTMLAttributes, ReactNode } from 'react';
import { useState } from 'react';
import { AlertCircle, Check, ChevronDown, ChevronRight, Copy } from 'lucide-react';
import { TEKS } from '../lib/teks';

export function Card({
  children,
  className = '',
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <section
      className={`bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-5 ${className}`}
    >
      {children}
    </section>
  );
}

type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost';
type ButtonSize = 'sm' | 'md' | 'lg';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  children: ReactNode;
}

const VARIANT_CLASSES: Record<ButtonVariant, string> = {
  primary:
    'bg-emerald-600 hover:bg-emerald-500 text-white border border-transparent disabled:bg-slate-200 disabled:text-slate-500 dark:disabled:bg-slate-800 dark:disabled:text-slate-400',
  secondary:
    'bg-slate-100 hover:bg-slate-200 text-slate-800 border border-slate-200 dark:bg-slate-800 dark:hover:bg-slate-700 dark:text-slate-100 dark:border-slate-700 disabled:opacity-50',
  danger:
    'bg-rose-600 hover:bg-rose-500 text-white border border-transparent disabled:bg-slate-200 disabled:text-slate-500 dark:disabled:bg-slate-800 dark:disabled:text-slate-400',
  ghost:
    'bg-transparent hover:bg-slate-100 text-slate-600 hover:text-slate-900 border border-transparent dark:text-slate-300 dark:hover:bg-slate-800 dark:hover:text-slate-100 disabled:opacity-50',
};

const SIZE_CLASSES: Record<ButtonSize, string> = {
  sm: 'px-2.5 py-1.5 text-xs',
  md: 'px-4 py-2 text-sm',
  lg: 'px-5 py-2.5 text-sm font-semibold',
};

export function Button({
  variant = 'secondary',
  size = 'md',
  className = '',
  type = 'button',
  children,
  ...rest
}: ButtonProps) {
  return (
    <button
      type={type}
      className={`inline-flex items-center justify-center gap-2 rounded-lg font-medium whitespace-nowrap shrink-0 transition-colors duration-150 cursor-pointer disabled:cursor-not-allowed focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-emerald-500 ${VARIANT_CLASSES[variant]} ${SIZE_CLASSES[size]} ${className}`}
      {...rest}
    >
      {children}
    </button>
  );
}

type BadgeTone = 'slate' | 'emerald' | 'amber' | 'rose';

const BADGE_CLASSES: Record<BadgeTone, string> = {
  slate:
    'bg-slate-100 text-slate-700 border-slate-200 dark:bg-slate-800 dark:text-slate-300 dark:border-slate-700',
  emerald:
    'bg-emerald-50 text-emerald-800 border-emerald-200 dark:bg-emerald-950/60 dark:text-emerald-300 dark:border-emerald-800/70',
  amber:
    'bg-amber-50 text-amber-800 border-amber-200 dark:bg-amber-950/60 dark:text-amber-300 dark:border-amber-800/70',
  rose:
    'bg-rose-50 text-rose-800 border-rose-200 dark:bg-rose-950/60 dark:text-rose-300 dark:border-rose-800/70',
};

export function Badge({
  tone = 'slate',
  children,
  className = '',
}: {
  tone?: BadgeTone;
  children: ReactNode;
  className?: string;
}) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 text-xs font-medium rounded-md border whitespace-nowrap shrink-0 ${BADGE_CLASSES[tone]} ${className}`}
    >
      {children}
    </span>
  );
}

const DOT_CLASSES: Record<BadgeTone, string> = {
  slate: 'bg-slate-400 dark:bg-slate-500',
  emerald: 'bg-emerald-500',
  amber: 'bg-amber-500',
  rose: 'bg-rose-500',
};

export function StatusDot({ tone }: { tone: BadgeTone }) {
  return (
    <span
      aria-hidden="true"
      className={`inline-block w-2.5 h-2.5 rounded-full shrink-0 ${DOT_CLASSES[tone]}`}
    />
  );
}

export function ErrorBox({
  pesan,
  detail,
  onSalinDetail,
}: {
  pesan: string;
  detail: string | null;
  onSalinDetail: (teks: string) => Promise<void>;
}) {
  const [terbuka, setTerbuka] = useState(false);
  const [tersalin, setTersalin] = useState(false);

  async function tanganiSalin(): Promise<void> {
    const teks = detail ? `${pesan}\n${detail}` : pesan;
    await onSalinDetail(teks);
    setTersalin(true);
    setTimeout(() => setTersalin(false), 1800);
  }

  return (
    <div
      role="alert"
      className="rounded-lg border border-rose-200 dark:border-rose-900/70 bg-rose-50 dark:bg-rose-950/40 p-3.5 text-sm text-rose-900 dark:text-rose-100"
    >
      <div className="flex items-start gap-2.5">
        <AlertCircle
          aria-hidden="true"
          className="w-4 h-4 text-rose-600 dark:text-rose-400 mt-0.5 shrink-0"
        />
        <div className="min-w-0 flex-1 space-y-2">
          <p className="font-medium leading-snug">{pesan}</p>
          {detail && (
            <div>
              <div className="flex flex-wrap items-center gap-3">
                <button
                  type="button"
                  aria-expanded={terbuka}
                  onClick={() => setTerbuka((v) => !v)}
                  className="inline-flex items-center gap-1 text-xs font-medium text-rose-700 dark:text-rose-300 hover:underline cursor-pointer focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-rose-500 rounded"
                >
                  {terbuka ? (
                    <ChevronDown className="w-3.5 h-3.5" aria-hidden="true" />
                  ) : (
                    <ChevronRight className="w-3.5 h-3.5" aria-hidden="true" />
                  )}
                  {TEKS.galat.detailTeknis}
                </button>
                <button
                  type="button"
                  onClick={() => void tanganiSalin()}
                  className="inline-flex items-center gap-1 text-xs font-medium text-rose-700 dark:text-rose-300 hover:underline cursor-pointer focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-rose-500 rounded"
                >
                  {tersalin ? (
                    <Check className="w-3.5 h-3.5" aria-hidden="true" />
                  ) : (
                    <Copy className="w-3.5 h-3.5" aria-hidden="true" />
                  )}
                  {tersalin ? TEKS.galat.tersalin : TEKS.galat.salinDetail}
                </button>
              </div>
              {terbuka && (
                <pre className="mt-2 p-2.5 rounded-md bg-rose-100/80 dark:bg-rose-950/80 border border-rose-200 dark:border-rose-900 font-mono text-xs text-rose-900 dark:text-rose-200 whitespace-pre-wrap [overflow-wrap:anywhere]">
                  {detail}
                </pre>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
