/**
 * El idioma de la interfaz y el cambio sin reiniciar.
 *
 * # Por qué un proxy y no una función
 *
 * Los componentes leen los textos como objetos (`textos.biblioteca.titulo`) y
 * muchos guardan un grupo al montarse (`const t = textos.cola`). Si `textos` fuera
 * el diccionario ya resuelto, esa referencia se quedaría en el idioma del
 * momento en que se tomó, y cambiar de idioma no repintaría nada. Cada grupo es,
 * en cambio, un proxy que **resuelve en el momento de leer** contra el idioma
 * actual, que es un `$state`: la lectura dentro de una plantilla o un `$derived`
 * queda suscrita al idioma, y al cambiarlo se repinta sola. Se descartó pasar a
 * `textos()` como función en los más de doscientos sitios que leen textos: el
 * resultado sería el mismo con un cambio mucho más grande y más fácil de olvidar
 * en un componente nuevo.
 *
 * # Por qué el español viaja siempre y los demás se cargan
 *
 * El español es la fuente (la forma de los demás sale de él) y aquello a lo que se
 * vuelve si algo no se reconoce, así que no puede esperar a nada. El inglés y el
 * portugués se cargan a demanda: llevarlos en el arranque pagaría, en cada
 * apertura, dos idiomas que la persona no eligió (`check-paquete`).
 *
 * # Sin destello al abrir
 *
 * `+layout.ts` espera a `iniciarIdioma` antes de pintar nada, con el idioma de la
 * caché local. Al cambiar de idioma con la aplicación abierta, la interfaz sigue
 * en el anterior hasta que llegan todos los diccionarios del nuevo: nunca pasa
 * por una pantalla a medio traducir.
 */

export const IDIOMAS = ['es', 'en', 'pt'] as const;
export type Idioma = (typeof IDIOMAS)[number];

/** Al que se vuelve cuando no hay nada elegido ni reconocible. */
export const IDIOMA_BASE: Idioma = 'es';

/** Nombre de cada idioma **en ese idioma**, que es como se reconoce el propio. */
export const NOMBRE_DEL_IDIOMA: Record<Idioma, string> = {
  es: 'Español',
  en: 'English',
  pt: 'Português'
};

/** La etiqueta BCP 47 que se pasa a `Intl` y a `toLocaleString`. */
const ETIQUETA: Record<Idioma, string> = { es: 'es', en: 'en', pt: 'pt-BR' };

/** Clave de la caché local: la comparten la ventana principal y el minirreproductor. */
const CACHE = 'ns-idioma';

export function esIdioma(valor: unknown): valor is Idioma {
  return typeof valor === 'string' && (IDIOMAS as readonly string[]).includes(valor);
}

type Cargable = Exclude<Idioma, 'es'>;
type Diccionario = Record<string, unknown>;

interface Registro {
  es: Diccionario;
  cargas: Record<Cargable, () => Promise<unknown>>;
  cargados: Partial<Record<Idioma, Diccionario>>;
}

const REGISTROS: Registro[] = [];

let actual = $state<Idioma>(IDIOMA_BASE);
/**
 * Sube cuando llega un diccionario tarde (el de una pantalla diferida que se abre
 * con otro idioma ya elegido): las lecturas hechas con el español de reserva se
 * vuelven a evaluar con el bueno.
 */
let version = $state(0);
/** El último pedido: un diccionario que llega tarde no pisa a otro elegido después. */
let pedido: Idioma = IDIOMA_BASE;

/** El idioma en uso. */
export function idioma(): Idioma {
  return actual;
}

/** La etiqueta para formatear números y fechas en el idioma en uso. */
export function localeActual(): string {
  return ETIQUETA[actual];
}

async function cargar(r: Registro, i: Idioma): Promise<void> {
  if (r.cargados[i]) return;
  r.cargados[i] = (await r.cargas[i as Cargable]()) as Diccionario;
}

function resolver(r: Registro, ruta: readonly PropertyKey[]): unknown {
  // Leer ambos estados suscribe a quien lee: al idioma y a las cargas tardías.
  const i = actual;
  void version;
  let nodo: unknown = r.cargados[i] ?? r.es;
  for (const clave of ruta) {
    if (nodo === null || typeof nodo !== 'object') return undefined;
    nodo = (nodo as Record<PropertyKey, unknown>)[clave];
  }
  return nodo;
}

function esGrupo(v: unknown): v is Diccionario {
  return v !== null && typeof v === 'object' && !Array.isArray(v);
}

function proxyDe(r: Registro, ruta: readonly PropertyKey[], cache: Map<string, object>): object {
  const llave = ruta.map(String).join('\u0000');
  const hecho = cache.get(llave);
  if (hecho) return hecho;
  const p = new Proxy(
    {},
    {
      get(_, clave) {
        const valor = resolver(r, [...ruta, clave]);
        return esGrupo(valor) ? proxyDe(r, [...ruta, clave], cache) : valor;
      },
      has(_, clave) {
        const g = resolver(r, ruta);
        return esGrupo(g) && clave in g;
      },
      ownKeys() {
        const g = resolver(r, ruta);
        return esGrupo(g) ? Reflect.ownKeys(g) : [];
      },
      getOwnPropertyDescriptor(_, clave) {
        const g = resolver(r, ruta);
        if (!esGrupo(g) || !(clave in g)) return undefined;
        const valor = (g as Record<PropertyKey, unknown>)[clave];
        return {
          enumerable: true,
          configurable: true,
          value: esGrupo(valor) ? proxyDe(r, [...ruta, clave], cache) : valor
        };
      }
    }
  );
  cache.set(llave, p);
  return p;
}

/**
 * Registra un diccionario y devuelve su cara reactiva.
 *
 * Si ya hay otro idioma elegido, su traducción se pide en el acto; mientras
 * llega, se ve el español, y al llegar se repinta.
 */
export function registrar<T extends object>(
  es: T,
  cargas: Record<Cargable, () => Promise<unknown>>
): T {
  const r: Registro = { es: es as unknown as Diccionario, cargas, cargados: { es: es as never } };
  REGISTROS.push(r);
  if (actual !== IDIOMA_BASE) {
    void cargar(r, actual).then(() => {
      version += 1;
    });
  }
  return proxyDe(r, [], new Map()) as T;
}

function aplicarEnDocumento(i: Idioma): void {
  if (typeof document !== 'undefined') document.documentElement.lang = ETIQUETA[i];
}

/**
 * Cambia el idioma de toda la interfaz, sin reiniciar.
 *
 * Espera a que estén todos los diccionarios del nuevo idioma antes de cambiar,
 * para no enseñar una pantalla mezclada.
 */
export async function usarIdioma(valor: unknown): Promise<Idioma> {
  const elegido = esIdioma(valor) ? valor : IDIOMA_BASE;
  pedido = elegido;
  await Promise.all(REGISTROS.map((r) => cargar(r, elegido)));
  if (pedido === elegido) {
    actual = elegido;
    aplicarEnDocumento(elegido);
    try {
      localStorage.setItem(CACHE, elegido);
    } catch {
      // Sin almacenamiento vale para esta sesión.
    }
  }
  return elegido;
}

/**
 * El idioma del sistema, si es uno de los que se hablan. Solo se usa la primera
 * vez, cuando todavía no hay ninguno elegido; a falta de coincidencia, inglés,
 * que es el que entiende más gente de quien no habla los otros dos.
 */
export function idiomaDelSistema(): Idioma {
  const pedidos =
    typeof navigator === 'undefined' ? [] : [...(navigator.languages ?? []), navigator.language];
  for (const etiqueta of pedidos) {
    const base = etiqueta?.slice(0, 2).toLowerCase();
    if (esIdioma(base)) return base;
  }
  return 'en';
}

/** El idioma de la caché local, o `null` si es la primera vez. */
export function idiomaEnCache(): Idioma | null {
  try {
    const v = localStorage.getItem(CACHE);
    return esIdioma(v) ? v : null;
  } catch {
    return null;
  }
}

/**
 * Lo que se hace antes del primer pintado: el idioma de la caché o, la primera
 * vez, el del sistema. Otra ventana de NB Sound (el minirreproductor) que cambie
 * de idioma lo escribe en la caché, y el evento `storage` lo trae aquí.
 */
export async function iniciarIdioma(): Promise<void> {
  await usarIdioma(idiomaEnCache() ?? idiomaDelSistema());
  if (typeof window !== 'undefined') {
    window.addEventListener('storage', (e) => {
      if (e.key === CACHE && esIdioma(e.newValue) && e.newValue !== actual) {
        void usarIdioma(e.newValue);
      }
    });
  }
}
