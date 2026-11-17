<!--
  @component
  La pestaña del DJ (ADR-007, ADR-018): DJ Privado y DJ Mashups en el
  mismo sitio. Un botón en la cabecera alterna la experiencia principal; el
  historial, a la derecha, es uno solo para las dos y no se desmonta al
  alternar. Cada experiencia, una vez abierta, se queda montada aunque esté
  oculta, y lo abierto y lo escrito viven en `estadoDj`: al volver de otra vista
  todo sigue como se dejó, y solo el usuario cierra una sesión o un mashup.
  Se carga aparte del arranque.
-->
<script lang="ts">
  import type { Elegida } from '#lib/core/ipc/mashups.ts';
  import { elegidaEn, modoDe, otro } from '#lib/ui/dj/experiencias.ts';
  import { estadoDj as e } from '#lib/ui/dj/estadoDj.svelte.ts';
  import { textosDj } from '#lib/i18n/dj.ts';
  import HistorialDj from '#lib/ui/dj/HistorialDj.svelte';
  import PedirSesion from '#lib/ui/dj/PedirSesion.svelte';
  import SesionActiva from '#lib/ui/dj/SesionActiva.svelte';
  import VistaMashups from '#lib/ui/dj/VistaMashups.svelte';
  import Icon from '#lib/ui/Icon.svelte';

  const t = textosDj;
  // Lo abierto vive fuera del componente: al volver de otra vista sigue igual.
  e.iniciar();
  let version = $state(0);

  function verSesion(id: number | null) {
    e.abrirSesion(id);
    version += 1;
  }

  function verMashup(id: number | null) {
    e.abrirMashup(id);
    version += 1;
  }

  /** Elegir en el historial abre la experiencia de lo elegido. */
  function ver(x: Elegida) {
    if (x.tipo === 'sesion') e.abrirSesion(x.id);
    else e.abrirMashup(x.id);
    e.cambiar(modoDe(x));
  }

  const elegida = $derived(elegidaEn(e.modo, e.sesion, e.mashup));
  const destino = $derived(otro(e.modo));
</script>

<section class="vista">
  <div class="principal">
    <header>
      <div class="titulos">
        <h1>{e.modo === 'privado' ? t.titulo : t.mashups.titulo}</h1>
        <p class="sub">{e.modo === 'privado' ? t.subtitulo : t.mashups.subtitulo}</p>
      </div>
      <button
        type="button"
        class="alternar"
        aria-label={destino === 'mashups' ? t.modos.irAMashups : t.modos.irAPrivado}
        title={destino === 'mashups' ? t.modos.irAMashups : t.modos.irAPrivado}
        onclick={() => e.cambiar(destino)}
      >
        <Icon nombre={destino === 'mashups' ? 'ecualizador' : 'dj'} tamano={18} />
        <span>{destino === 'mashups' ? t.modos.aMashups : t.modos.aPrivado}</span>
        <Icon nombre="adelante" tamano={16} />
      </button>
    </header>

    <div class="experiencia" hidden={e.modo !== 'privado'}>
      <!-- La sesión que se ve va arriba: debajo del formulario, al pedirla había que
           bajar para saber que ya sonaba. -->
      {#if e.sesion !== null}
        <SesionActiva sesionId={e.sesion} oncambio={verSesion} />
      {/if}
      <PedirSesion onpedida={verSesion} />
    </div>

    {#if e.mashupsAbierto}
      <div class="experiencia" hidden={e.modo !== 'mashups'}>
        <VistaMashups mashupId={e.mashup} onver={verMashup} />
      </div>
    {/if}
  </div>
  <aside class="lateral" aria-labelledby="t-historial">
    <HistorialDj seleccionada={elegida} onver={ver} {version} />
  </aside>
</section>

<style>
  .vista {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 22rem;
    gap: var(--ns-espacio-5);
    align-items: start;
  }

  .principal {
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-4);
    min-width: 0;
  }

  header {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--ns-espacio-3);
  }

  .titulos {
    display: flex;
    flex: 1 1 18rem;
    flex-direction: column;
    gap: var(--ns-espacio-1);
    min-width: 0;
  }

  /* Un solo botón, siempre en el mismo sitio, que dice adónde lleva. */
  .alternar {
    display: inline-flex;
    flex: 0 0 auto;
    align-items: center;
    gap: var(--ns-espacio-2);
    height: 2.5rem;
    padding: 0 var(--ns-espacio-4);
    border: 1px solid var(--m3c-outline-variant);
    border-radius: var(--ns-radio-pleno);
    background: var(--m3c-surface-container);
    color: var(--m3c-on-surface);
    font: inherit;
    font-size: var(--ns-texto-sm);
    font-weight: 600;
    cursor: pointer;
    transition:
      background var(--ns-duracion-corta) var(--ns-curva),
      border-color var(--ns-duracion-corta) var(--ns-curva);
  }

  .alternar:hover {
    border-color: var(--m3c-primary);
    background: var(--m3c-surface-container-high);
  }

  .alternar :global(svg:first-child) {
    color: var(--m3c-primary);
  }

  .experiencia {
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-4);
    animation: aparecer var(--ns-duracion-corta) var(--ns-curva);
  }

  .experiencia[hidden] {
    display: none;
  }

  @keyframes aparecer {
    from {
      opacity: 0;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .experiencia {
      animation: none;
    }
  }

  h1 {
    margin: 0;
    font-size: var(--ns-texto-xl);
  }

  .sub {
    margin: 0;
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-sm);
  }

  .lateral {
    position: sticky;
    top: 0;
  }

  @container carcasa (max-width: 70rem) {
    .vista {
      grid-template-columns: minmax(0, 1fr);
    }

    .lateral {
      position: static;
    }
  }
</style>
