/**
 * Punto único de entrada a los textos del arranque. Los componentes importan de
 * `#lib/i18n/index.ts` y nunca de los archivos de idioma.
 */
import { es } from './es';
import { registrar } from './idioma.svelte';

export type { Textos } from './es';
export {
  esIdioma,
  IDIOMA_BASE,
  IDIOMAS,
  type Idioma,
  idioma,
  idiomaDelSistema,
  idiomaEnCache,
  iniciarIdioma,
  localeActual,
  NOMBRE_DEL_IDIOMA,
  usarIdioma
} from './idioma.svelte';

export const textos = registrar(es, {
  en: () => import('./en').then((m) => m.en),
  pt: () => import('./pt').then((m) => m.pt)
});
