import { iniciarIdioma } from '#lib/i18n/index.ts';

// Aplicación de página única dentro de Tauri: sin renderizado en servidor, y
// todo se prerenderiza contra el `fallback` del adaptador estático.
export const ssr = false;
export const prerender = true;

// SvelteKit no pinta nada hasta que `load` termina: así el primer fotograma ya
// sale en el idioma elegido, sin pasar por el español.
export async function load(): Promise<void> {
  await iniciarIdioma();
}
