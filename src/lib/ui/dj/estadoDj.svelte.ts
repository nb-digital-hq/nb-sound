/**
 * Lo que la pestaña del DJ tiene abierto, fuera de sus componentes: la
 * experiencia que se ve, la sesión y el mashup abiertos y lo escrito en los dos
 * formularios. Al irse a otra vista y volver, todo sigue como se dejó, como una
 * ventana minimizada; solo el usuario cierra una sesión o un mashup.
 *
 * El trabajo en curso (una sesión que se construye, un mashup que se genera)
 * vive en el motor y sigue sin la vista; esto solo recuerda qué mirar al volver.
 *
 * Vive en el módulo de la pestaña, que ya se carga aparte del arranque. Entre
 * aperturas de la aplicación se recuerdan la experiencia y lo abierto, en el
 * almacenamiento del webview (es de este equipo); si no está disponible, se
 * empieza por lo que suena.
 */
import type { Estilo } from '#lib/core/ipc/mashups.ts';
import { reproductor } from '#lib/core/reproductor.svelte.ts';
import { type Modo, modoInicial } from './experiencias.ts';

const CLAVE = 'nb-sound:dj-abierto';
const CLAVE_MODO = 'nb-sound:dj-modo';

interface Guardado {
  modo?: Modo;
  sesion?: number | null;
  mashup?: number | null;
}

function leer(): Guardado {
  try {
    const g = JSON.parse(localStorage.getItem(CLAVE) ?? '{}') as Guardado;
    return typeof g === 'object' && g !== null ? g : {};
  } catch {
    return {};
  }
}

function modoViejo(): string | null {
  try {
    return localStorage.getItem(CLAVE_MODO);
  } catch {
    return null;
  }
}

const entero = (v: unknown): number | null =>
  typeof v === 'number' && Number.isInteger(v) ? v : null;

class EstadoDj {
  #iniciado = false;
  modo = $state<Modo>('privado');
  sesion = $state<number | null>(null);
  mashup = $state<number | null>(null);
  /** Los mashups se montan la primera vez que se abren y ya no se desmontan. */
  mashupsAbierto = $state(false);
  pedirSesion = $state({ texto: '', minutos: 60 });
  pedirMashup = $state({ origen: 0, canciones: 8, minutos: 5, estilo: 'equilibrado' as Estilo });

  /** La primera vez en esta apertura: lo guardado, y lo que suena por encima. */
  iniciar() {
    if (this.#iniciado) return;
    this.#iniciado = true;
    const g = leer();
    const sonando = reproductor.instantanea;
    this.modo = modoInicial(sonando, g.modo ?? modoViejo());
    this.sesion = sonando.dj?.sesionId ?? entero(g.sesion);
    this.mashup = sonando.mashupId ?? entero(g.mashup);
    this.mashupsAbierto = this.modo === 'mashups';
  }

  cambiar(modo: Modo) {
    this.modo = modo;
    if (modo === 'mashups') this.mashupsAbierto = true;
    this.guardar();
  }

  abrirSesion(id: number | null) {
    this.sesion = id;
    this.guardar();
  }

  abrirMashup(id: number | null) {
    this.mashup = id;
    this.guardar();
  }

  guardar() {
    try {
      localStorage.setItem(
        CLAVE,
        JSON.stringify({ modo: this.modo, sesion: this.sesion, mashup: this.mashup })
      );
    } catch {
      // Sin almacenamiento local solo se pierde recordarlo al reabrir la app.
    }
  }
}

export const estadoDj = new EstadoDj();
