/** Punto de entrada a los saludos de Inicio, cargados a demanda. */
import { esSaludos } from './es-saludos';
import { registrar } from './idioma.svelte';

export type { TextosSaludos } from './es-saludos';

export const textosSaludos = registrar(esSaludos, {
  en: () => import('./en-saludos').then((m) => m.enSaludos),
  pt: () => import('./pt-saludos').then((m) => m.ptSaludos)
});
