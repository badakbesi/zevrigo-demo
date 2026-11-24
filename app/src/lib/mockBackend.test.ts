import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createMockBackend } from './mockBackend';

const ALAMAT_SAH =
  'rzvrg1qjdyrjv4zg5ykef5f7ww7vpysezk543yj6w50tmnwkmutjtjxcekqz5cp69';

describe('mockBackend', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('menjalankan alur nyalakan node -> sinkron -> menambang -> menemukan blok', async () => {
    const api = createMockBackend();

    const awal = await api.keadaanNode();
    expect(awal.status).toBe('mati');

    await expect(api.mulaiTambang(ALAMAT_SAH, 120)).rejects.toThrow();

    await api.nyalakanNode();
    expect((await api.keadaanNode()).status).toBe('menyala');

    await vi.advanceTimersByTimeAsync(1100);
    expect((await api.keadaanNode()).status).toBe('menyambung');

    await vi.advanceTimersByTimeAsync(2100);
    expect((await api.keadaanNode()).status).toBe('mengejar');

    await vi.advanceTimersByTimeAsync(10500);
    const sinkron = await api.keadaanNode();
    expect(sinkron.status).toBe('sinkron');
    expect(sinkron.tinggiLokal).toBe(sinkron.tinggiJaringan);
    expect(sinkron.rekan).toBeGreaterThanOrEqual(2);

    await api.mulaiTambang(ALAMAT_SAH, 60);
    expect((await api.keadaanTambang()).status).toBe('menambang');

    await vi.advanceTimersByTimeAsync(3200);
    const sesudahBlok = await api.keadaanTambang();
    expect(sesudahBlok.blokSesiIni).toBe(1);
    expect(sesudahBlok.blokTerakhir).toHaveLength(1);
    expect(sesudahBlok.blokTerakhir[0].idBlok).toMatch(/^[0-9a-f]{64}$/);
    expect(sesudahBlok.status).toBe('menunggu');

    await api.berhentiTambang();
    expect((await api.keadaanTambang()).status).toBe('berhenti');

    await api.hentikanNode();
    expect((await api.keadaanNode()).status).toBe('mati');
  });

  it('memvalidasi alamat secara tepat', async () => {
    const api = createMockBackend();
    expect(await api.periksaAlamat('')).toEqual({ sah: false, alasan: 'kosong' });
    expect(await api.periksaAlamat('tzvrg1abc')).toEqual({
      sah: false,
      alasan: 'jaringan-lain',
    });
    expect(await api.periksaAlamat('rzvrg1qqqqq')).toEqual({
      sah: false,
      alasan: 'format',
    });
    expect(await api.periksaAlamat(ALAMAT_SAH)).toEqual({
      sah: true,
      alasan: null,
    });
  });
});
