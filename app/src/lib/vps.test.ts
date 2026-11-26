import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { perluKunci } from './storeVps';
import {
  createMockVpsApi,
  createVpsBackend,
  formatDurasi,
  formatMb,
  formatZvrg,
  keGalat,
  persenKuota,
  type VpsApi,
} from './vps';

describe('tiruan mode VPS', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('menjalankan tahap pertama kali sampai siap, kunci, buka, ekspor, impor', async () => {
    const api = createMockVpsApi();
    expect((await api.keadaan()).tahap).toBe('baru');
    await expect(api.buatSandi('pendek')).rejects.toMatchObject({ kode: 'masukan' });
    expect((await api.buatSandi('sandi-aplikasi')).tahap).toBe('sambung');

    const sambung = api.sambung('203.0.113.7', 22, 'root', 'rahasia');
    await vi.advanceTimersByTimeAsync(700);
    const info = await sambung;
    expect(info.tahap).toBe('pasang');
    expect(info.sidik).toMatch(/^SHA256:/);
    expect(info.masalah).toEqual([]);
    expect(info.pemeriksaan?.root).toBe(true);

    const pasang = api.pasang();
    await vi.advanceTimersByTimeAsync(2600);
    expect((await pasang).tahap).toBe('amankan');
    expect((await api.status()).loginPassword).toBe('yes');

    const amankan = api.amankan();
    await vi.advanceTimersByTimeAsync(600);
    expect((await amankan).tahap).toBe('siap');
    expect((await api.status()).loginPassword).toBe('no');

    expect((await api.kunci()).tahap).toBe('terkunci');
    await expect(api.rpc('info', {})).rejects.toMatchObject({ kode: 'terkunci' });
    const salah = expect(api.buka('sandi-salah-sekali')).rejects.toMatchObject({ kode: 'sandi_salah' });
    await vi.advanceTimersByTimeAsync(400);
    await salah;
    const buka = api.buka('sandi-aplikasi');
    await vi.advanceTimersByTimeAsync(400);
    expect((await buka).tahap).toBe('siap');

    const teks = await api.ekspor();
    expect(teks).not.toContain('sandi-aplikasi');
    expect((await api.lupakan()).tahap).toBe('baru');
    await expect(api.impor(teks, 'sandi-salah-sekali')).rejects.toMatchObject({ kode: 'sandi_salah' });
    await expect(api.impor('bukan data', 'sandi-aplikasi')).rejects.toMatchObject({ kode: 'brankas' });
    const lagi = await api.impor(teks, 'sandi-aplikasi');
    expect(lagi.tahap).toBe('siap');
    expect(lagi.host).toBe('203.0.113.7');

    const cpu = api.kendali({ jenis: 'batas_cpu', persen: 30 });
    await vi.advanceTimersByTimeAsync(500);
    await cpu;
    expect((await api.status()).kuotaCpu).toBe('300ms');
    expect(await api.rpc<{ versiNode: string }>('info', {})).toMatchObject({ versiNode: expect.any(String) });
    await expect(api.rpc('jalankan', {})).rejects.toMatchObject({ kode: 'pengendali' });
  });

  it('menolak password VPS yang salah dan bisa dibatalkan', async () => {
    const api = createMockVpsApi();
    await api.buatSandi('sandi-aplikasi');
    const salah = expect(api.sambung('203.0.113.7', 22, 'zvrg', 'salah')).rejects.toMatchObject({
      kode: 'masuk_ditolak',
    });
    await vi.advanceTimersByTimeAsync(700);
    await salah;
    expect((await api.batal()).tahap).toBe('baru');
  });
});

describe('createVpsBackend', () => {
  it('memetakan tombol kartu ke metode pengendali di VPS', async () => {
    const panggilan: [string, Record<string, unknown>][] = [];
    const tiruan = createMockVpsApi();
    const api: VpsApi = {
      ...tiruan,
      rpc: async <T>(metode: string, params: Record<string, unknown>): Promise<T> => {
        panggilan.push([metode, params]);
        const jawaban =
          metode === 'muat_pengaturan' ? { alamatHadiah: 'rzvrg1abc', jedaDetik: 300, tema: 'gelap' } : null;
        return jawaban as T;
      },
    };
    const b = createVpsBackend(api);
    await b.mulaiTambang('rzvrg1abc', 60);
    await b.logTerbaru(null);
    await b.periksaAlamat('rzvrg1abc');
    const p = await b.muatPengaturan();
    expect(p).toEqual({ alamatHadiah: 'rzvrg1abc', jedaDetik: 300, tema: 'sistem' });
    await b.simpanPengaturan({ alamatHadiah: 'rzvrg1abc', jedaDetik: 120, tema: 'terang' });
    await b.berhentiTambang();
    expect(panggilan).toEqual([
      ['mulai_tambang', { alamat: 'rzvrg1abc', jeda: 60 }],
      ['log_terbaru', { sejak: null }],
      ['periksa_alamat', { alamat: 'rzvrg1abc' }],
      ['muat_pengaturan', {}],
      ['simpan_pengaturan', { pengaturan: { alamatHadiah: 'rzvrg1abc', jedaDetik: 120, tema: 'sistem' } }],
      ['berhenti_tambang', {}],
    ]);
    let tutup = false;
    const henti = b.saatAkanTutup(() => {
      tutup = true;
    });
    henti();
    expect(tutup).toBe(false);
  });
});

describe('format mode VPS', () => {
  it('memformat saldo, ukuran, lama menyala, batas CPU, galat, kunci otomatis', () => {
    expect(formatZvrg('13.777828687378155611')).toBe('13,77782868');
    expect(formatZvrg('1234567.5')).toBe('1.234.567,5');
    expect(formatZvrg('12.000000000000000000')).toBe('12');
    expect(formatZvrg('0')).toBe('0');
    expect(formatMb(512)).toBe('512 MB');
    expect(formatMb(3911)).toBe('3,8 GB');
    expect(formatMb(Number.NaN)).toBe('-');
    expect(formatDurasi(691618)).toBe('8 hari 0 jam');
    expect(formatDurasi(3725)).toBe('1 jam 2 menit');
    expect(formatDurasi(59)).toBe('0 menit');
    expect(persenKuota('500ms')).toBe(50);
    expect(persenKuota('300ms')).toBe(30);
    expect(persenKuota('infinity')).toBeNull();
    expect(keGalat({ kode: 'sidik_berubah', pesan: 'sidik kunci VPS berubah' })).toEqual({
      kode: 'sidik_berubah',
      pesan: 'sidik kunci VPS berubah',
    });
    expect(keGalat('teks')).toEqual({ kode: 'sistem', pesan: 'teks' });
    expect(keGalat(new Error('putus')).pesan).toBe('putus');
    expect(perluKunci(null, 1_000_000)).toBe(false);
    expect(perluKunci(0, 59_999)).toBe(false);
    expect(perluKunci(0, 60_000)).toBe(true);
  });
});
