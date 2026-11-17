/**
 * Punto de entrada a los textos de lo que se carga aparte del arranque. Solo lo
 * importan módulos diferidos: importarlo desde el arranque lo metería en él.
 */
import { esAparte } from './es-aparte';
import { registrar } from './idioma.svelte';

export type { TextosAparte } from './es-aparte';

export const textosAparte = registrar(esAparte, {
  en: () => import('./en-aparte').then((m) => m.enAparte),
  pt: () => import('./pt-aparte').then((m) => m.ptAparte)
});
