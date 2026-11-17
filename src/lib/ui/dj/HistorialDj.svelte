<!--
  @component
  El historial del DJ: las sesiones del DJ Privado y los
  mashups en una sola lista, de lo más reciente a lo más antiguo, con una
  búsqueda que no distingue tildes ni mayúsculas. Es el mismo recuadro para las
  dos experiencias: la vista lo monta una vez y no lo desmonta al alternar.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { alCambiarDj, djEliminar } from '#lib/core/ipc/dj.ts';
  import {
    alTerminarMashup,
    djHistorialCompleto,
    type Elegida,
    type EntradaHistorialDj,
    mashupEliminar
  } from '#lib/core/ipc/mashups.ts';
  import { formatoTiempo } from '#lib/core/reproductor.svelte.ts';
  import { textosDj } from '#lib/i18n/dj.ts';
  import { localeActual } from '#lib/i18n/index.ts';
  import Icon from '#lib/ui/Icon.svelte';

  interface Props {
    seleccionada: Elegida | null;
    onver: (e: Elegida) => void;
    /** Para volver a leerlo cuando la vista sabe que cambió. */
    version: number;
  }

  let { seleccionada, onver, version }: Props = $props();
  const t = textosDj.historial;
  const tm = textosDj.mashups;

  let texto = $state('');
  let filas = $state<EntradaHistorialDj[]>([]);
  let leidas = $state(false);
  let propia = $state(0);

  $effect(() => {
    const consulta = texto;
    void version;
    void propia;
    void djHistorialCompleto(consulta).then((f) => {
      if (consulta === texto) {
        filas = f;
        leidas = true;
      }
    });
  });

  const dejarDj = alCambiarDj(() => (propia += 1));
  const dejarMashup = alTerminarMashup(() => (propia += 1));
  onDestroy(() => {
    void dejarDj.then((f) => f());
    void dejarMashup.then((f) => f());
  });

  async function eliminar(f: EntradaHistorialDj) {
    if (f.tipo === 'sesion') await djEliminar(f.id);
    else await mashupEliminar(f.id);
    propia += 1;
  }

  function fecha(iso: string): string {
    const d = new Date(iso);
    return Number.isNaN(d.getTime())
      ? ''
      : d.toLocaleDateString(localeActual(), { dateStyle: 'medium' });
  }

  const elegida = (f: EntradaHistorialDj) =>
    seleccionada?.tipo === f.tipo && seleccionada.id === f.id;
</script>

<section class="historial" aria-labelledby="t-historial">
  <h2 id="t-historial">{t.titulo}</h2>
  <label class="busqueda">
    <Icon nombre="buscar" tamano={18} />
    <input type="search" placeholder={t.buscar} aria-label={t.buscar} bind:value={texto} />
  </label>

  {#if leidas && filas.length === 0 && texto.trim()}
    <p class="vacio">{t.nada}</p>
  {/if}

  <ul>
    {#each filas as f (`${f.tipo}-${f.id}`)}
      <li class:elegida={elegida(f)}>
        <button
          type="button"
          class="fila"
          aria-current={elegida(f) ? 'true' : undefined}
          onclick={() => onver({ tipo: f.tipo, id: f.id })}
        >
          <span class="tipo" title={t.tipos[f.tipo]}>
            <Icon
              nombre={f.tipo === 'sesion' ? 'dj' : 'ecualizador'}
              tamano={18}
              etiqueta={t.tipos[f.tipo]}
            />
          </span>
          <span class="texto">
            {#if f.tipo === 'sesion'}
              <strong>{f.peticion}</strong>
              <small>
                {t.estados[f.estado] ?? f.estado} · {fecha(f.creadaEn)} ·
                {t.resumen(f.totalPistas, formatoTiempo(f.duracionSeg))}
              </small>
            {:else}
              <strong>{f.titulo}</strong>
              <small>
                {tm.estados[f.estado] ?? f.estado} · {fecha(f.creadoEn)}
                {#if f.estado === 'lista'}
                  · {tm.resumen(f.canciones, formatoTiempo(f.duracionSeg))}
                {/if}
              </small>
            {/if}
          </span>
        </button>
        <button
          type="button"
          class="icono"
          aria-label={f.tipo === 'sesion'
            ? t.eliminarSesion(f.peticion)
            : t.eliminarMashup(f.titulo)}
          title={t.eliminar}
          onclick={() => eliminar(f)}
        >
          <Icon nombre="borrar" tamano={16} />
        </button>
      </li>
    {/each}
  </ul>

  <!-- Ocupa lo que la lista deja libre: con pocas entradas se ve entero, y a
       medida que se llena se encoge hasta desaparecer (consulta de contenedor). -->
  {#if leidas && !texto.trim()}
    <div class="adorno" aria-hidden={filas.length > 0}>
      <div class="adorno-contenido">
        <span class="disco"><Icon nombre="dj" tamano={40} /></span>
        <strong>{filas.length === 0 ? t.vacio : t.pocas}</strong>
        <small>{t.adornoTexto}</small>
      </div>
    </div>
  {/if}
</section>

<style>
  .historial {
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-3);
    padding: var(--ns-espacio-4);
    border: 1px solid var(--m3c-outline-variant);
    border-radius: var(--ns-radio-xl);
    background: var(--m3c-surface-container);
    /* Toda la altura del área de contenido: antes medía lo que su lista y dejaba
       media columna vacía. Las mismas medidas que la carcasa (AppShell). */
    height: calc(
      100vh - var(--ns-alto-superior) - var(--ns-alto-reproductor) - var(--ns-espacio-2) * 2 -
        var(--ns-espacio-6)
    );
    min-height: 24rem;
  }

  h2 {
    margin: 0;
    font-size: var(--ns-texto-lg);
  }

  .busqueda {
    display: flex;
    align-items: center;
    gap: var(--ns-espacio-2);
    height: 2.25rem;
    padding: 0 var(--ns-espacio-4);
    border: 1px solid var(--m3c-outline-variant);
    border-radius: var(--ns-radio-pleno);
    background: var(--m3c-surface-container-high);
    color: var(--m3c-on-surface-variant);
  }

  .busqueda:focus-within {
    border-color: var(--m3c-primary);
  }

  /* La caja entera ya marca el foco; el contorno del propio campo, dentro,
     dibujaba un segundo marco. */
  .busqueda input:focus-visible {
    outline-color: transparent;
  }

  .busqueda input {
    flex: 1;
    border: none;
    background: none;
    color: var(--m3c-on-surface);
    font: inherit;
    font-size: var(--ns-texto-xs);
  }

  .vacio {
    margin: 0;
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-sm);
  }

  ul {
    display: flex;
    flex: 0 1 auto;
    flex-direction: column;
    gap: var(--ns-espacio-1);
    min-height: 0;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .adorno {
    display: grid;
    flex: 1 1 0;
    place-items: center;
    min-height: 0;
    overflow: hidden;
    border-radius: var(--ns-radio-lg);
    background: radial-gradient(
      circle at 50% 40%,
      color-mix(in srgb, var(--m3c-primary) 14%, transparent),
      transparent 70%
    );
    container-type: size;
  }

  .adorno-contenido {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--ns-espacio-2);
    max-width: 16rem;
    padding: var(--ns-espacio-4);
    text-align: center;
  }

  .adorno-contenido small {
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-xs);
    line-height: 1.5;
  }

  .disco {
    display: grid;
    place-items: center;
    width: 5.5rem;
    height: 5.5rem;
    margin-bottom: var(--ns-espacio-2);
    border: 1px solid color-mix(in srgb, var(--m3c-primary) 40%, transparent);
    border-radius: var(--ns-radio-pleno);
    background: var(--m3c-surface-container-high);
    color: var(--m3c-primary);
    box-shadow: 0 0 0 0.5rem color-mix(in srgb, var(--m3c-primary) 8%, transparent);
  }

  /* Poco sitio: primero se va el texto pequeño, luego todo. */
  @container (max-height: 15rem) {
    .adorno-contenido small {
      display: none;
    }
  }

  @container (max-height: 10rem) {
    .adorno-contenido {
      display: none;
    }
  }

  li {
    display: flex;
    align-items: center;
    border-radius: var(--ns-radio-lg);
    transition: background 120ms ease;
  }

  li:hover:not(.elegida) {
    background: var(--m3c-surface-container-high);
  }

  li.elegida {
    background: var(--m3c-secondary-container);
    color: var(--m3c-on-secondary-container);
  }

  .fila {
    display: flex;
    flex: 1;
    align-items: center;
    gap: var(--ns-espacio-3);
    min-width: 0;
    padding: var(--ns-espacio-2) var(--ns-espacio-3);
    border: none;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .tipo {
    display: grid;
    flex: 0 0 auto;
    place-items: center;
    width: 2rem;
    height: 2rem;
    border-radius: var(--ns-radio-pleno);
    background: var(--m3c-surface-container-high);
    color: var(--m3c-primary);
  }

  li.elegida .tipo {
    background: color-mix(in srgb, var(--m3c-on-secondary-container) 12%, transparent);
    color: inherit;
  }

  .texto {
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-1);
    min-width: 0;
  }

  .fila strong,
  .fila small {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .fila small {
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-xs);
  }

  .icono {
    display: grid;
    place-items: center;
    width: 2.5rem;
    height: 2.5rem;
    border: none;
    border-radius: var(--ns-radio-pleno);
    background: none;
    color: var(--m3c-on-surface-variant);
    cursor: pointer;
  }

  .icono:hover {
    background: var(--m3c-surface-container);
  }
</style>
