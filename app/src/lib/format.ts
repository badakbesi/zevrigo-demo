export function formatAngka(nilai: number | null | undefined): string {
  if (nilai === null || nilai === undefined || !Number.isFinite(nilai)) {
    return '-';
  }
  const bulat = Math.trunc(nilai);
  const tanda = bulat < 0 ? '-' : '';
  const digit = Math.abs(bulat).toString();
  const hasil = digit.replace(/\B(?=(\d{3})+(?!\d))/g, '.');
  return `${tanda}${hasil}`;
}

export function formatWaktu(epochMs: number): string {
  if (!Number.isFinite(epochMs)) {
    return '--:--:--';
  }
  const d = new Date(epochMs);
  const jam = String(d.getHours()).padStart(2, '0');
  const menit = String(d.getMinutes()).padStart(2, '0');
  const detik = String(d.getSeconds()).padStart(2, '0');
  return `${jam}:${menit}:${detik}`;
}

export function formatHitungMundur(detik: number | null | undefined): string {
  if (detik === null || detik === undefined || !Number.isFinite(detik)) {
    return '00:00';
  }
  const aman = Math.max(0, Math.floor(detik));
  const menit = Math.floor(aman / 60);
  const sisa = aman % 60;
  return `${String(menit).padStart(2, '0')}:${String(sisa).padStart(2, '0')}`;
}

export function singkatIdBlok(idBlok: string): string {
  return idBlok.slice(0, 8);
}
