import type { HasilPeriksaAlamat } from './backend';

const KARAKTER_BECH32 = /^[qpzry9x8gf2tvdw0s3jn54khce6mua7l]+$/;
const PANJANG_MIN_WAJAR = 14;
const PANJANG_MAX_WAJAR = 90;

export function periksaAlamatAwal(masukan: string): HasilPeriksaAlamat {
  const alamat = masukan.trim();
  if (alamat.length === 0) {
    return { sah: false, alasan: 'kosong' };
  }

  const hurufKecil = alamat.toLowerCase();
  if (hurufKecil.startsWith('tzvrg1') || hurufKecil.startsWith('zvrg1')) {
    return { sah: false, alasan: 'jaringan-lain' };
  }

  if (alamat !== hurufKecil || !alamat.startsWith('rzvrg1')) {
    return { sah: false, alasan: 'format' };
  }

  if (alamat.length < PANJANG_MIN_WAJAR || alamat.length > PANJANG_MAX_WAJAR) {
    return { sah: false, alasan: 'format' };
  }

  const bagianData = alamat.slice(6);
  if (!KARAKTER_BECH32.test(bagianData)) {
    return { sah: false, alasan: 'format' };
  }

  return { sah: true, alasan: null };
}
