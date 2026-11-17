import { describe, expect, it } from 'vitest';
import type { Capa } from '#lib/core/ipc/mashups.ts';
import { caja, carriles, elegidaEn, modoDe, modoInicial, otro } from './experiencias.ts';

const capa = (material: Capa['material'], destino: number, compases = 8): Capa => ({
  pistaId: 1,
  material,
  origen: 0,
  compases,
  destino,
  tempo: 1,
  semitonos: 0,
  gananciaDb: 0,
  entrada: 0.125,
  salida: 0.125,
  transicion: 'cruce',
  gravesFuera: false
});

describe('la pestaña del DJ con dos experiencias', () => {
  it('abre en lo que suena y, si no suena nada, en la última usada', () => {
    expect(modoInicial({ mashupId: 3, dj: null }, 'privado')).toBe('mashups');
    expect(modoInicial({ mashupId: null, dj: { sesionId: 1 } }, 'mashups')).toBe('privado');
    expect(modoInicial({ mashupId: null, dj: null }, 'mashups')).toBe('mashups');
    expect(modoInicial({ dj: null }, null)).toBe('privado');
    expect(modoInicial({ dj: null }, 'basura')).toBe('privado');
  });

  it('el botón siempre lleva a la otra experiencia', () => {
    expect(otro('privado')).toBe('mashups');
    expect(otro('mashups')).toBe('privado');
  });

  it('elegir en el historial abre la experiencia de lo elegido', () => {
    expect(modoDe({ tipo: 'sesion', id: 1 })).toBe('privado');
    expect(modoDe({ tipo: 'mashup', id: 1 })).toBe('mashups');
  });

  it('el historial solo marca lo elegido en la experiencia visible y conserva lo de la otra', () => {
    expect(elegidaEn('privado', 4, 9)).toEqual({ tipo: 'sesion', id: 4 });
    expect(elegidaEn('mashups', 4, 9)).toEqual({ tipo: 'mashup', id: 9 });
    expect(elegidaEn('mashups', 4, null)).toBeNull();
  });
});

describe('la línea de tiempo de un mashup', () => {
  it('reparte las capas en carriles por papel y en orden', () => {
    const c = carriles([
      capa('voz', 8),
      capa('instrumental', 8),
      capa('mezcla', 0),
      capa('sin_bateria', 16),
      capa('bateria', 16)
    ]);
    expect(c.map((x) => x.clave)).toEqual(['base', 'voz', 'bateria']);
    expect(c[0]?.capas.map((x) => x.destino)).toEqual([0, 8, 16]);
  });

  it('no pinta carriles vacíos', () => {
    expect(carriles([capa('mezcla', 0)]).map((x) => x.clave)).toEqual(['base']);
    expect(carriles([])).toEqual([]);
  });

  it('las cajas caben siempre en la línea', () => {
    expect(caja(capa('mezcla', 8, 8), 32)).toEqual({ izquierda: 25, ancho: 25 });
    expect(caja(capa('mezcla', 30, 8), 32).izquierda + caja(capa('mezcla', 30, 8), 32).ancho).toBe(
      100
    );
    expect(caja(capa('mezcla', 0), 0)).toEqual({ izquierda: 0, ancho: 0 });
  });
});
