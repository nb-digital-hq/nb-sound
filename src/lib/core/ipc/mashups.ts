/**
 * El puente de DJ Mashups (ADR-018). Como el del DJ Privado, solo lo
 * importa la vista del DJ, que se carga aparte del arranque.
 */

import type { UnlistenFn } from '@tauri-apps/api/event';
import type {
  Capa,
  Degradacion,
  EntradaHistorialDj,
  EstadoMashups,
  Estilo,
  Etapa,
  FilaMashup,
  Material,
  PeticionMashup,
  Plan,
  Progreso,
  VistaMashup
} from '#lib/bindings/index.ts';

import { call, escuchar } from './index';

export type {
  Capa,
  Degradacion,
  EntradaHistorialDj,
  EstadoMashups,
  Estilo,
  Etapa,
  FilaMashup,
  Material,
  PeticionMashup,
  Plan,
  Progreso,
  VistaMashup
};

/** Lo que se ve en la vista del DJ: una sesión o un mashup. */
export interface Elegida {
  tipo: 'sesion' | 'mashup';
  id: number;
}

/** Sesiones y mashups en una sola lista, del más reciente al más antiguo. */
export function djHistorialCompleto(texto: string): Promise<EntradaHistorialDj[]> {
  return call<EntradaHistorialDj[]>('dj_historial_completo', { texto });
}

export function mashupEstado(): Promise<EstadoMashups> {
  return call<EstadoMashups>('mashup_estado');
}

/** Empieza a generar; devuelve el id sin esperar a que termine. */
export function mashupGenerar(peticion: PeticionMashup): Promise<number> {
  return call<number>('mashup_generar', { peticion });
}

/** La misma petición con otra semilla. */
export function mashupVariante(mashupId: number): Promise<number> {
  return call<number>('mashup_variante', { mashupId });
}

export function mashupCancelar(mashupId: number): Promise<void> {
  return call<void>('mashup_cancelar', { mashupId });
}

export function mashupVista(mashupId: number): Promise<VistaMashup> {
  return call<VistaMashup>('mashup_vista', { mashupId });
}

export function mashupReproducir(mashupId: number, desdeSeg = 0): Promise<void> {
  return call<void>('mashup_reproducir', { mashupId, desdeSeg });
}

export function mashupEliminar(mashupId: number): Promise<void> {
  return call<void>('mashup_eliminar', { mashupId });
}

/** Abre el selector del sistema; `false` si se canceló. */
export function mashupExportar(mashupId: number): Promise<boolean> {
  return call<boolean>('mashup_exportar', { mashupId });
}

/** Avance real de la generación en curso: como mucho uno por punto porcentual. */
export function alProgresoMashup(cada: (p: Progreso) => void): Promise<UnlistenFn> {
  return escuchar<Progreso>('mashup:progreso', cada);
}

/** Una generación terminó, bien, mal o cancelada. */
export function alTerminarMashup(cada: (mashupId: number) => void): Promise<UnlistenFn> {
  return escuchar<number>('mashup:terminado', cada);
}
