import { describe, expect, it } from 'vitest';
import {
  formatAngka,
  formatHitungMundur,
  formatWaktu,
  singkatIdBlok,
} from './format';

describe('formatAngka', () => {
  it('memformat ribuan dengan titik', () => {
    expect(formatAngka(0)).toBe('0');
    expect(formatAngka(999)).toBe('999');
    expect(formatAngka(1820)).toBe('1.820');
    expect(formatAngka(1943)).toBe('1.943');
    expect(formatAngka(1234567)).toBe('1.234.567');
  });

  it('menangani nilai kosong atau tidak valid', () => {
    expect(formatAngka(null)).toBe('-');
    expect(formatAngka(undefined)).toBe('-');
    expect(formatAngka(Number.NaN)).toBe('-');
  });
});

describe('formatWaktu', () => {
  it('memformat jam:menit:detik dua digit', () => {
    const contoh = new Date(2026, 10, 23, 9, 5, 7).getTime();
    expect(formatWaktu(contoh)).toBe('09:05:07');
  });
});

describe('formatHitungMundur', () => {
  it('memformat detik ke mm:ss', () => {
    expect(formatHitungMundur(0)).toBe('00:00');
    expect(formatHitungMundur(65)).toBe('01:05');
    expect(formatHitungMundur(120)).toBe('02:00');
    expect(formatHitungMundur(-5)).toBe('00:00');
    expect(formatHitungMundur(null)).toBe('00:00');
  });
});

describe('singkatIdBlok', () => {
  it('mengambil 8 karakter pertama', () => {
    expect(singkatIdBlok('a1b2c3d4e5f60708')).toBe('a1b2c3d4');
  });
});
