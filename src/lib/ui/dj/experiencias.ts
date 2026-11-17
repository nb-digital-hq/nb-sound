/**
 * Las reglas de la pestaña del DJ con dos experiencias: cuál se ve,
 * qué está elegido en el historial y cómo se reparte un plan en carriles. Van
 * aparte de los componentes para poder probarlas sin montar la interfaz.
 */
import type { Capa, Elegida } from '#lib/core/ipc/mashups.ts';

export type Modo = 'privado' | 'mashups';

/** Lo que suena manda; si no suena nada del DJ, la última experiencia usada. */
export function modoInicial(
  sonando: { mashupId?: number | null; dj: unknown },
  guardado: string | null
): Modo {
  if (sonando.mashupId != null) return 'mashups';
  if (sonando.dj) return 'privado';
  return guardado === 'mashups' ? 'mashups' : 'privado';
}

/** La otra experiencia: la que anuncia el botón. */
export function otro(modo: Modo): Modo {
  return modo === 'privado' ? 'mashups' : 'privado';
}

/** La experiencia en la que se abre algo elegido en el historial. */
export function modoDe(e: Elegida): Modo {
  return e.tipo === 'sesion' ? 'privado' : 'mashups';
}

/** Lo que el historial marca: solo lo de la experiencia que se ve. */
export function elegidaEn(
  modo: Modo,
  sesion: number | null,
  mashup: number | null
): Elegida | null {
  if (modo === 'privado') return sesion === null ? null : { tipo: 'sesion', id: sesion };
  return mashup === null ? null : { tipo: 'mashup', id: mashup };
}

export type ClaveCarril = 'base' | 'voz' | 'bateria';

/** Las capas por carril, en orden de destino; los carriles vacíos no se pintan. */
export function carriles(capas: readonly Capa[]): { clave: ClaveCarril; capas: Capa[] }[] {
  const de = (c: Capa): ClaveCarril =>
    c.material === 'voz' ? 'voz' : c.material === 'bateria' ? 'bateria' : 'base';
  const orden: ClaveCarril[] = ['base', 'voz', 'bateria'];
  return orden
    .map((clave) => ({
      clave,
      capas: capas.filter((c) => de(c) === clave).sort((a, b) => a.destino - b.destino)
    }))
    .filter((c) => c.capas.length > 0);
}

/** Posición y ancho de una capa en porcentaje del total, acotados a [0, 100]. */
export function caja(capa: Capa, compases: number): { izquierda: number; ancho: number } {
  if (compases <= 0) return { izquierda: 0, ancho: 0 };
  const izquierda = Math.min(100, Math.max(0, (capa.destino / compases) * 100));
  const ancho = Math.min(100 - izquierda, Math.max(0, (capa.compases / compases) * 100));
  return { izquierda, ancho };
}
