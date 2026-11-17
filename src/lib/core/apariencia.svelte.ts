/**
 * La apariencia que eligió el usuario: tema, tipografía y escala.
 *
 * Las preferencias viven en la base, que solo se lee de forma asíncrona. Para que
 * el primer fotograma ya tenga el tema y no parpadee con el de fábrica, las
 * variables del último tema aplicado se guardan en `localStorage` y se aplican al
 * arrancar, antes de preguntar al runtime; si la respuesta trae otro tema, se
 * corrige.
 */

import { cambios } from '#lib/core/cambios.svelte.ts';
import { type Apariencia, apariencia, guardarPreferencia } from '#lib/core/ipc/index.ts';
import { esIdioma, idioma, usarIdioma } from '#lib/i18n/index.ts';
import { aplicarTema, aplicarVariables, TEMA_DE_FABRICA } from '#lib/ui/tema.ts';

const CACHE = 'ns-apariencia';
const BARRA_COMPACTA = 'ns-barra-lateral-compacta';

interface EnCache {
  tema: string;
  variables: Record<string, string>;
  oscuro: boolean;
  tipografia: string;
  escala: number;
}

const DE_FABRICA: Apariencia = {
  tema: TEMA_DE_FABRICA,
  tipografia: 'Inter',
  escala: 100,
  colorPorPortada: true,
  nombreUsuario: null,
  idioma: null
};

function aplicarTipografia(raiz: HTMLElement, familia: string, escala: number): void {
  raiz.style.setProperty(
    '--ns-font-ui',
    `'${familia}', system-ui, -apple-system, 'Segoe UI', sans-serif`
  );
  // La escala (`ui_scale`) mueve el tamaño base: todo está en rem.
  raiz.style.fontSize = `${escala}%`;
}

class EstadoDeApariencia {
  actual = $state<Apariencia>(DE_FABRICA);
  /**
   * La barra lateral en riel por decisión del usuario. Es de este equipo
   * y no de la biblioteca: en otra pantalla puede caber entera, así que vive en
   * `localStorage` como la caché del tema y no en las preferencias.
   */
  barraLateralCompacta = $state(false);

  alternarBarraLateral(): void {
    this.barraLateralCompacta = !this.barraLateralCompacta;
    try {
      localStorage.setItem(BARRA_COMPACTA, String(this.barraLateralCompacta));
    } catch {
      // Sin almacenamiento, vale para esta sesión.
    }
  }

  /** Lo que haya en caché, sin esperar a nada. Se llama antes del primer pintado. */
  aplicarCache(raiz: HTMLElement): void {
    try {
      this.barraLateralCompacta = localStorage.getItem(BARRA_COMPACTA) === 'true';
      const guardado = localStorage.getItem(CACHE);
      if (!guardado) return;
      const c = JSON.parse(guardado) as EnCache;
      aplicarVariables(raiz, c.variables, c.oscuro);
      aplicarTipografia(raiz, c.tipografia, c.escala);
    } catch {
      // Una caché rota no impide arrancar: llega la de verdad enseguida.
    }
  }

  /** Lee las preferencias y las aplica. */
  async cargar(raiz: HTMLElement): Promise<void> {
    const a = await apariencia().catch(() => DE_FABRICA);
    await this.#aplicar(raiz, a);
    await this.#sincronizarIdioma(a);
  }

  /**
   * La base manda sobre la caché local: es la que sobrevive a una copia de
   * seguridad restaurada en otro equipo. La primera vez no hay nada guardado y se
   * guarda el idioma con el que ya se pintó (el del sistema), para que el runtime
   * (bandeja, diálogos, listas que nombra él) hable el mismo que la interfaz.
   */
  async #sincronizarIdioma(a: Apariencia): Promise<void> {
    if (esIdioma(a.idioma)) {
      if (a.idioma !== idioma()) await usarIdioma(a.idioma);
      return;
    }
    const enUso = idioma();
    this.actual = { ...this.actual, idioma: enUso };
    await guardarPreferencia('idioma', enUso).catch(() => {});
  }

  async #aplicar(raiz: HTMLElement, a: Apariencia): Promise<void> {
    this.actual = a;
    const variables =
      (await aplicarTema(raiz, a.tema)) ?? (await aplicarTema(raiz, DE_FABRICA.tema));
    aplicarTipografia(raiz, a.tipografia, a.escala);
    if (!variables) return;
    try {
      const c: EnCache = {
        tema: a.tema,
        variables,
        oscuro: raiz.dataset.esquema === 'oscuro',
        tipografia: a.tipografia,
        escala: a.escala
      };
      localStorage.setItem(CACHE, JSON.stringify(c));
    } catch {
      // Sin almacenamiento, el próximo arranque parpadea un instante; nada más.
    }
  }

  /** Cambia una preferencia, la guarda y la aplica al momento. */
  async cambiar<
    K extends 'tema' | 'tipografia' | 'escala' | 'colorPorPortada' | 'nombreUsuario' | 'idioma'
  >(clave: K, valor: Apariencia[K]): Promise<void> {
    const claves: Record<K, string> = {
      tema: 'tema',
      tipografia: 'tipografia',
      escala: 'escala',
      colorPorPortada: 'color_por_portada',
      nombreUsuario: 'nombre_usuario',
      idioma: 'idioma'
    } as Record<K, string>;
    await guardarPreferencia(claves[clave], valor);
    if (clave === 'idioma') {
      await usarIdioma(valor);
      this.actual = { ...this.actual, idioma: valor as Apariencia['idioma'] };
      // Lo que nombra el runtime (las playlists que crea, sus etiquetas) ya está
      // traducido en la base: las vistas lo vuelven a pedir.
      cambios.avisar();
      return;
    }
    await this.#aplicar(document.documentElement, { ...this.actual, [clave]: valor });
  }
}

export const aparienciaActual = new EstadoDeApariencia();
