<!--
  @component
  Pedir un mashup: de dónde salen las canciones, cuántas, cuánto dura
  y con qué estilo. Lo demás (qué fragmentos, qué capas, tempo y tonalidad) lo
  decide el compositor. Dice antes de empezar si faltan el separador de voces o
  canciones analizadas, para no prometer lo que no se puede hacer.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { playlistsListar, type PlaylistResumen } from '#lib/core/ipc/index.ts';
  import {
    alTerminarMashup,
    type Estilo,
    type EstadoMashups,
    mashupEstado,
    mashupGenerar
  } from '#lib/core/ipc/mashups.ts';
  import { estadoDj } from '#lib/ui/dj/estadoDj.svelte.ts';
  import { textosDj } from '#lib/i18n/dj.ts';
  import Icon from '#lib/ui/Icon.svelte';
  import MenuSelector from '#lib/ui/MenuSelector.svelte';

  interface Props {
    onempezado: (mashupId: number) => void;
  }

  let { onempezado }: Props = $props();
  const t = textosDj.mashups;
  const CANCIONES = [4, 8, 16, 32, 64, 100];
  const DURACIONES = [3, 5, 10, 20];
  const ESTILOS: Estilo[] = ['equilibrado', 'fiesta', 'suave', 'relampago'];

  let estado = $state<EstadoMashups | null>(null);
  let playlists = $state<PlaylistResumen[]>([]);
  // Lo elegido sobrevive a irse de la pestaña: vive en `estadoDj`.
  const borrador = estadoDj.pedirMashup;
  let origen = $state<number>(borrador.origen);
  let canciones = $state(borrador.canciones);
  let minutos = $state(borrador.minutos);
  let estilo = $state<Estilo>(borrador.estilo);
  $effect(() => {
    Object.assign(borrador, { origen, canciones, minutos, estilo });
  });
  let enviando = $state(false);
  let aviso = $state<string | null>(null);

  function leer() {
    void mashupEstado().then((e) => (estado = e));
  }
  leer();
  void playlistsListar({ texto: null, categoria: null, orden: 'nombreAsc' }).then(
    (p) => (playlists = p.filter((x) => x.numPistas >= 2))
  );
  const dejar = alTerminarMashup(() => leer());
  onDestroy(() => void dejar.then((f) => f()));

  const opcionesOrigen = $derived([
    { valor: 0, etiqueta: t.toda, icono: 'biblioteca' as const },
    ...playlists.map((p) => ({
      valor: p.id,
      etiqueta: t.playlist(p.nombre),
      icono: 'playlists' as const
    }))
  ]);
  const ocupado = $derived(enviando || Boolean(estado?.enCurso));
  const sinMaterial = $derived(estado !== null && estado.analizadas < 2);

  // Relámpago es el único estilo con frases de cuatro compases: muchas
  // canciones en pocos minutos solo caben así. Se propone al elegir cantidad o
  // duración, y el usuario puede cambiarlo después.
  function proponerEstilo() {
    if (canciones >= 32 && minutos <= 5) estilo = 'relampago';
  }

  async function crear() {
    enviando = true;
    aviso = null;
    try {
      const id = await mashupGenerar({
        duracionSeg: minutos * 60,
        canciones,
        estilo,
        semilla: null,
        pistas: [],
        playlistId: origen === 0 ? null : origen
      });
      onempezado(id);
      leer();
    } catch {
      aviso = estado?.enCurso ? t.ocupado : t.error;
    } finally {
      enviando = false;
    }
  }
</script>

<form
  class="pedir"
  onsubmit={(e) => {
    e.preventDefault();
    void crear();
  }}
>
  <div class="fila">
    <span class="rotulo">{t.origen}</span>
    <MenuSelector
      opciones={opcionesOrigen}
      bind:valor={origen}
      ariaLabel={t.origen}
      deshabilitado={ocupado}
    />
  </div>

  <div class="fila" role="group" aria-label={t.cuantas}>
    <span class="rotulo">{t.cuantas}</span>
    {#each CANCIONES as n (n)}
      <button
        type="button"
        class="ficha"
        class:activa={canciones === n}
        aria-pressed={canciones === n}
        disabled={ocupado}
        onclick={() => {
          canciones = n;
          proponerEstilo();
        }}
      >
        {n}
      </button>
    {/each}
  </div>

  <div class="fila" role="group" aria-label={t.duracion}>
    <span class="rotulo">{t.duracion}</span>
    {#each DURACIONES as d (d)}
      <button
        type="button"
        class="ficha"
        class:activa={minutos === d}
        aria-pressed={minutos === d}
        disabled={ocupado}
        onclick={() => {
          minutos = d;
          proponerEstilo();
        }}
      >
        {t.minutos(d)}
      </button>
    {/each}
  </div>

  <div class="estilos" role="group" aria-labelledby="rotulo-estilo">
    <span class="rotulo" id="rotulo-estilo">{t.estilo}</span>
    {#each ESTILOS as e (e)}
      <button
        type="button"
        class="estilo"
        class:activa={estilo === e}
        aria-pressed={estilo === e}
        disabled={ocupado}
        onclick={() => (estilo = e)}
      >
        <strong>{t.estilos[e].nombre}</strong>
        <small>{t.estilos[e].ayuda}</small>
      </button>
    {/each}
  </div>

  {#if estado}
    <p class="nota">
      <Icon nombre="informacion" tamano={16} />
      <span>{sinMaterial ? t.pocasAnalizadas : estado.conStems ? t.conStems : t.sinStems}</span>
    </p>
  {/if}

  <div class="pie">
    {#if aviso}<p class="aviso" role="status">{aviso}</p>{/if}
    <button type="submit" class="boton" disabled={ocupado || sinMaterial || estado === null}>
      <Icon nombre={enviando ? 'cargando' : 'ecualizador'} tamano={16} />
      {t.crear}
    </button>
  </div>
</form>

<style>
  .pedir {
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-4);
    padding: var(--ns-espacio-5);
    border: 1px solid var(--m3c-outline-variant);
    border-radius: var(--ns-radio-xl);
    background: var(--m3c-surface-container);
    box-shadow: 0 4px 16px var(--m3c-shadow);
  }

  .fila {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ns-espacio-2);
  }

  .rotulo {
    min-width: 5.5rem;
    margin-right: var(--ns-espacio-2);
    padding: 0;
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-xs);
    font-weight: 600;
  }

  .ficha {
    min-width: 2.75rem;
    height: 2rem;
    padding: 0 var(--ns-espacio-3);
    border: 1px solid var(--m3c-outline-variant);
    border-radius: var(--ns-radio-pleno);
    background: none;
    color: var(--m3c-on-surface);
    font: inherit;
    font-size: var(--ns-texto-xs);
    font-variant-numeric: tabular-nums;
    cursor: pointer;
  }

  .ficha:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .ficha.activa {
    border-color: var(--m3c-primary);
    background: var(--m3c-secondary-container);
    color: var(--m3c-on-secondary-container);
    font-weight: 700;
  }

  .estilos {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));
    gap: var(--ns-espacio-2);
  }

  .estilos .rotulo {
    grid-column: 1 / -1;
  }

  .estilo {
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-1);
    padding: var(--ns-espacio-3);
    border: 1px solid var(--m3c-outline-variant);
    border-radius: var(--ns-radio-lg);
    background: var(--m3c-surface-container-high);
    color: var(--m3c-on-surface);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: border-color var(--ns-duracion-corta) var(--ns-curva);
  }

  .estilo:hover:not(:disabled) {
    border-color: var(--m3c-outline);
  }

  .estilo.activa {
    border-color: var(--m3c-primary);
    background: var(--m3c-secondary-container);
    color: var(--m3c-on-secondary-container);
  }

  .estilo strong {
    font-size: var(--ns-texto-sm);
  }

  .estilo small {
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-xs);
    line-height: 1.4;
  }

  .estilo.activa small {
    color: inherit;
  }

  .estilo:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .nota {
    display: flex;
    align-items: flex-start;
    gap: var(--ns-espacio-2);
    margin: 0;
    padding: var(--ns-espacio-2) var(--ns-espacio-3);
    border-radius: var(--ns-radio-md);
    background: var(--m3c-surface-container-high);
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-sm);
    line-height: 1.45;
  }

  .nota :global(svg) {
    flex: 0 0 auto;
    margin-top: 0.15rem;
  }

  .pie {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ns-espacio-3);
  }

  .aviso {
    flex: 1 1 14rem;
    margin: 0;
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-sm);
  }

  .boton {
    display: inline-flex;
    align-items: center;
    gap: var(--ns-espacio-2);
    height: 2.5rem;
    margin-left: auto;
    padding: 0 var(--ns-espacio-5);
    border: none;
    border-radius: var(--ns-radio-pleno);
    background: var(--m3c-primary);
    color: var(--m3c-on-primary);
    font: inherit;
    font-size: var(--ns-texto-sm);
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 2px 8px var(--m3c-shadow);
    transition: filter var(--ns-duracion-corta) var(--ns-curva);
  }

  .boton:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .boton:disabled {
    opacity: 0.38;
    cursor: default;
  }
</style>
