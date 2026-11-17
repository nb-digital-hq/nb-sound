/**
 * Punto de entrada a los textos del DJ. Solo lo importa la vista del DJ.
 */
import { esDj } from './es-dj';
import { registrar } from './idioma.svelte';

export type { TextosDj } from './es-dj';

export const textosDj = registrar(esDj, {
  en: () => import('./en-dj').then((m) => m.enDj),
  pt: () => import('./pt-dj').then((m) => m.ptDj)
});
