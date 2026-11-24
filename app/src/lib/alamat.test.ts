import { describe, expect, it } from 'vitest';
import { periksaAlamatAwal } from './alamat';

const ALAMAT_CONTOH_SAH =
  'rzvrg1qjdyrjv4zg5ykef5f7ww7vpysezk543yj6w50tmnwkmutjtjxcekqz5cp69';

describe('periksaAlamatAwal', () => {
  it('menerima alamat rzvrg1 yang sah', () => {
    expect(periksaAlamatAwal(ALAMAT_CONTOH_SAH)).toEqual({
      sah: true,
      alasan: null,
    });
  });

  it('menolak masukan kosong', () => {
    expect(periksaAlamatAwal('')).toEqual({ sah: false, alasan: 'kosong' });
    expect(periksaAlamatAwal('   ')).toEqual({ sah: false, alasan: 'kosong' });
  });

  it('menolak alamat jaringan lain', () => {
    expect(
      periksaAlamatAwal(
        'tzvrg1qjdyrjv4zg5ykef5f7ww7vpysezk543yj6w50tmnwkmutjtjxcekqz5cp69',
      ),
    ).toEqual({ sah: false, alasan: 'jaringan-lain' });
    expect(
      periksaAlamatAwal(
        'zvrg1qjdyrjv4zg5ykef5f7ww7vpysezk543yj6w50tmnwkmutjtjxcekqz5cp69',
      ),
    ).toEqual({ sah: false, alasan: 'jaringan-lain' });
  });

  it('menolak huruf besar, karakter bukan bech32, dan panjang tidak wajar', () => {
    expect(periksaAlamatAwal(ALAMAT_CONTOH_SAH.toUpperCase())).toEqual({
      sah: false,
      alasan: 'format',
    });
    expect(periksaAlamatAwal('rzvrg1abc!@#')).toEqual({
      sah: false,
      alasan: 'format',
    });
    expect(periksaAlamatAwal('rzvrg1qqqq')).toEqual({
      sah: false,
      alasan: 'format',
    });
  });
});
