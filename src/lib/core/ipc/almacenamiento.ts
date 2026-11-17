/**
 * El puente de Ajustes > Almacenamiento. Solo lo importa esa sección, que se
 * carga aparte del arranque.
 */

import type { CategoriaAlmacen, IdAlmacen, InformeAlmacen } from '#lib/bindings/index.ts';

import { call } from './index';

export type { CategoriaAlmacen, IdAlmacen, InformeAlmacen };

/** Lo que ocupa cada categoría, su límite y lo que ocuparía con toda la biblioteca. */
export function almacenamientoInforme(): Promise<InformeAlmacen> {
  return call<InformeAlmacen>('almacenamiento_informe');
}

/** El límite de las separaciones, en bytes; poda hasta quedar debajo. */
export function almacenamientoFijarTope(bytes: number): Promise<void> {
  return call<void>('almacenamiento_fijar_tope', { bytes });
}

/** Vacía una categoría que se puede rehacer; devuelve los bytes liberados. */
export function almacenamientoLiberar(categoria: IdAlmacen): Promise<number> {
  return call<number>('almacenamiento_liberar', { categoria });
}
