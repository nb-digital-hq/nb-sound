/**
 * Punto de entrada a los textos de A ciegas. Solo lo importa su vista.
 */
import { esACiegas } from './es-aciegas';
import { registrar } from './idioma.svelte';

export type { TextosACiegas } from './es-aciegas';

export const textosACiegas = registrar(esACiegas, {
  en: () => import('./en-aciegas').then((m) => m.enACiegas),
  pt: () => import('./pt-aciegas').then((m) => m.ptACiegas)
});
