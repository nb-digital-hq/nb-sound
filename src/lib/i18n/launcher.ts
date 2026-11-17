/** Punto de entrada a los textos del acceso de NB Launcher, cargados a demanda. */
import { esLauncher } from './es-launcher';
import { registrar } from './idioma.svelte';

export type { TextosLauncher } from './es-launcher';

export const textosLauncher = registrar(esLauncher, {
  en: () => import('./en-launcher').then((m) => m.enLauncher),
  pt: () => import('./pt-launcher').then((m) => m.ptLauncher)
});
