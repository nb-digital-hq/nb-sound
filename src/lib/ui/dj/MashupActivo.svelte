<!--
  @component
  Un mashup: mientras se prepara, su progreso real por etapas y la
  opción de cancelar; listo, sus acciones (escuchar, otra variante, exportar),
  lo que el compositor tuvo que ceder y una línea de tiempo simplificada con qué
  canción suena en cada carril y cuándo. Pulsar un bloque escucha desde ahí.
  No es un editor: la composición la decide el compositor.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { urlPortada } from '#lib/core/ipc/index.ts';
  import {
    alProgresoMashup,
    alTerminarMashup,
    type Capa,
    type Degradacion,
    mashupCancelar,
    mashupEliminar,
    mashupExportar,
    mashupReproducir,
    mashupVariante,
    mashupVista,
    type Progreso,
    type VistaMashup
  } from '#lib/core/ipc/mashups.ts';
  import { notificar } from '#lib/core/notificaciones.svelte.ts';
  import { formatoTiempo, reproductor } from '#lib/core/reproductor.svelte.ts';
  import { textosDj } from '#lib/i18n/dj.ts';
  import Icon from '#lib/ui/Icon.svelte';
  import { caja, carriles as repartir } from './experiencias.ts';

  interface Props {
    mashupId: number;
    /** Otro mashup pasa a verse (una variante) o ninguno (al borrarlo). */
    oncambio: (mashupId: number | null) => void;
  }

  let { mashupId, oncambio }: Props = $props();
  const t = textosDj.mashups;
  const NOTAS = [
    'Do',
    'Do sostenido',
    'Re',
    'Re sostenido',
    'Mi',
    'Fa',
    'Fa sostenido',
    'Sol',
    'Sol sostenido',
    'La',
    'La sostenido',
    'Si'
  ];

  let m = $state<VistaMashup | null>(null);
  let progreso = $state<Progreso | null>(null);

  function cargar() {
    const id = mashupId;
    void mashupVista(id)
      .then((v) => {
        if (id === mashupId) m = v;
      })
      // Recordado pero ya borrado: se cierra en vez de quedarse en blanco.
      .catch(() => {
        if (id === mashupId) oncambio(null);
      });
  }

  // Otro mashup en el mismo componente: se lee el nuevo y se olvida el
  // progreso del anterior.
  $effect(() => {
    void mashupId;
    progreso = null;
    cargar();
  });

  const dejarProgreso = alProgresoMashup((p) => {
    if (p.mashupId === mashupId) progreso = p;
  });
  const dejarFin = alTerminarMashup((id) => {
    if (id === mashupId) {
      progreso = null;
      cargar();
    }
  });
  onDestroy(() => {
    void dejarProgreso.then((f) => f());
    void dejarFin.then((f) => f());
  });

  const i = $derived(reproductor.instantanea);
  const sonando = $derived(i.mashupId === mashupId);
  const enPausa = $derived(!sonando || i.estado === 'pausado');
  const plan = $derived(m?.plan ?? null);
  const compas = $derived(plan ? (4 * 60) / plan.bpm : 1);
  const total = $derived(plan ? plan.compases * compas : 0);

  /** Cada canción con su color, por orden de aparición; los colores son tokens. */
  const indice = $derived(new Map((m?.pistas ?? []).map((p, k) => [p.pistaId, k])));
  const titulo = (c: Capa) => m?.pistas[indice.get(c.pistaId) ?? -1]?.titulo ?? '';

  const carriles = $derived(
    repartir(plan?.capas ?? []).map((c) => ({ ...c, nombre: t.carriles[c.clave] }))
  );

  function degradacion(d: Degradacion): string {
    switch (d.tipo) {
      case 'sin_stems':
        return t.degradaciones.sin_stems;
      case 'menos_canciones':
        return t.degradaciones.menos_canciones(d.pedidas, d.usadas);
      case 'frases_prolongadas':
        return t.degradaciones.frases_prolongadas(d.frases);
      case 'mas_corto':
        return t.degradaciones.mas_corto(formatoTiempo(d.obtenidosSeg));
      case 'no_caben':
        return t.degradaciones.no_caben(d.pedidas, d.caben, formatoTiempo(d.minimoSeg));
    }
  }

  function escuchar() {
    if (sonando) void reproductor.pausarReanudar();
    else void mashupReproducir(mashupId, 0);
  }

  async function variante() {
    try {
      oncambio(await mashupVariante(mashupId));
    } catch {
      notificar.informacion(t.ocupado);
    }
  }

  async function exportar() {
    try {
      if (await mashupExportar(mashupId)) notificar.exito(t.exportado);
    } catch (e) {
      notificar.error((e as Error).message === 'PATH_NOT_ALLOWED' ? t.noExportable : t.error);
    }
  }

  async function eliminar() {
    await mashupEliminar(mashupId);
    oncambio(null);
  }
</script>

{#if m}
  <section class="mashup" aria-labelledby="t-mashup">
    <button
      type="button"
      class="cerrar-vista"
      aria-label={textosDj.cerrar}
      title={textosDj.cerrar}
      onclick={() => oncambio(null)}
    >
      <Icon nombre="cerrar" tamano={18} />
    </button>
    <header class="cabecera">
      <!-- Un mashup no es ninguna de sus canciones: su portada es su icono. -->
      <div class="portada" aria-hidden="true">
        <Icon nombre="ecualizador" tamano={56} />
      </div>
      <div class="datos">
        <span class="estado">
          {sonando
            ? enPausa
              ? textosDj.sesion.enPausa
              : textosDj.sesion.sonando
            : (t.estados[m.estado] ?? m.estado)}
        </span>
        <h2 id="t-mashup">{m.titulo}</h2>
        <p class="cifras">
          {t.estilos[m.peticion.estilo].nombre}
          {#if plan}
            · {t.resumen(m.pistas.length, formatoTiempo(total))} · {t.tempo(plan.bpm)} ·
            {t.tonalidad(NOTAS[plan.tonica] ?? '', plan.menor)}
          {/if}
        </p>
      </div>
    </header>

    {#if m.estado === 'generando'}
      {@const f = progreso?.fraccion ?? 0}
      <div class="progreso" role="status" aria-live="polite">
        <div class="linea-progreso">
          <strong>{progreso ? (t.etapas[progreso.etapa] ?? '') : t.etapas.eligiendo}</strong>
          {#if progreso && progreso.total > 0}
            <span class="unidades">{t.deUnidades(progreso.hechas, progreso.total)}</span>
          {/if}
          <span class="pct">{t.porcentaje(f)}</span>
        </div>
        <div
          class="pista-barra"
          role="progressbar"
          aria-label={t.etapas[progreso?.etapa ?? 'eligiendo']}
          aria-valuemin="0"
          aria-valuemax="100"
          aria-valuenow={Math.round(f * 100)}
        >
          <span class="relleno" style:width={`${f * 100}%`}></span>
        </div>
        <button type="button" class="pildora" onclick={() => mashupCancelar(mashupId)}>
          <Icon nombre="cerrar" tamano={16} />{t.cancelar}
        </button>
      </div>
    {:else if m.estado !== 'lista'}
      <div class="problema" role="status">
        <Icon nombre={m.estado === 'cancelada' ? 'informacion' : 'advertencia'} tamano={20} />
        <p>
          {m.estado === 'cancelada'
            ? t.estados.cancelada
            : (t.errores[m.error ?? 'INTERNAL'] ?? t.errores.INTERNAL)}
        </p>
        <button type="button" class="pildora" onclick={variante}>
          <Icon nombre="reintentar" tamano={16} />{t.reintentar}
        </button>
        <button type="button" class="pildora" onclick={eliminar}>
          <Icon nombre="borrar" tamano={16} />{t.eliminar}
        </button>
      </div>
    {:else}
      <div class="acciones">
        <button
          type="button"
          class="reproducir"
          aria-label={enPausa ? t.escuchar : t.pausar}
          title={enPausa ? t.escuchar : t.pausar}
          onclick={escuchar}
        >
          <Icon nombre={enPausa ? 'reproducir' : 'pausa'} tamano={26} />
        </button>
        <button type="button" class="pildora" onclick={variante}>
          <Icon nombre="reintentar" tamano={16} />{t.variante}
        </button>
        <button type="button" class="pildora" onclick={exportar}>
          <Icon nombre="exportar" tamano={16} />{t.exportar}
        </button>
        <button type="button" class="pildora discreta" onclick={eliminar}>
          <Icon nombre="borrar" tamano={16} />{t.eliminar}
        </button>
      </div>

      {#if plan && plan.degradaciones.length > 0}
        <ul class="cedido">
          {#each plan.degradaciones as d (d.tipo)}
            <li><Icon nombre="informacion" tamano={16} />{degradacion(d)}</li>
          {/each}
        </ul>
      {/if}

      {#if plan}
        <h3>{t.lineaDeTiempo}</h3>
        <div class="linea">
          {#each carriles as c (c.clave)}
            <div class="carril">
              <span class="nombre-carril">{c.nombre}</span>
              <div class="pista-carril">
                {#each c.capas as capa, k (`${capa.pistaId}-${capa.destino}-${k}`)}
                  {@const n = indice.get(capa.pistaId) ?? 0}
                  <button
                    type="button"
                    class="bloque c{n % 6}"
                    style:left={`${caja(capa, plan.compases).izquierda}%`}
                    style:width={`${caja(capa, plan.compases).ancho}%`}
                    title={`${titulo(capa)} · ${t.materiales[capa.material] ?? capa.material}`}
                    aria-label={t.bloque(
                      titulo(capa),
                      t.materiales[capa.material] ?? capa.material,
                      formatoTiempo(capa.destino * compas)
                    )}
                    onclick={() => mashupReproducir(mashupId, capa.destino * compas)}
                  >
                    <span>{titulo(capa)}</span>
                  </button>
                {/each}
              </div>
            </div>
          {/each}
          {#if sonando && total > 0}
            <span
              class="cabezal"
              aria-hidden="true"
              style:left={`calc(var(--ancho-nombre) + (100% - var(--ancho-nombre)) * ${Math.min(1, reproductor.posicion / total)})`}
            ></span>
          {/if}
          <div class="eje" aria-hidden="true">
            <span>0:00</span>
            <span>{formatoTiempo(total / 2)}</span>
            <span>{formatoTiempo(total)}</span>
          </div>
        </div>
      {/if}

      <h3>{t.pistas}</h3>
      <ol class="pistas">
        {#each m.pistas as p, k (k)}
          {@const primera = plan?.capas.find((c) => c.pistaId === p.pistaId)}
          <li>
            <button
              type="button"
              class="pista"
              disabled={!primera}
              onclick={() => primera && mashupReproducir(mashupId, primera.destino * compas)}
            >
              <span class="punto c{k % 6}" aria-hidden="true"></span>
              {#if p.pistaId !== null}
                <img
                  class="mini"
                  src={urlPortada('pista', p.pistaId, 64)}
                  alt=""
                  loading="lazy"
                  decoding="async"
                />
              {:else}
                <span class="mini"></span>
              {/if}
              <span class="datos-pista">
                <strong>{p.titulo}</strong>
                <small>{p.artista}</small>
              </span>
              <span class="duracion">{primera ? formatoTiempo(primera.destino * compas) : ''}</span>
            </button>
          </li>
        {/each}
      </ol>
    {/if}
  </section>
{/if}

<style>
  .cerrar-vista {
    position: absolute;
    top: var(--ns-espacio-3);
    right: var(--ns-espacio-3);
    display: grid;
    place-items: center;
    width: 2.25rem;
    height: 2.25rem;
    border: 0;
    border-radius: var(--ns-radio-pleno);
    background: transparent;
    color: var(--m3c-on-surface-variant);
    cursor: pointer;
  }

  .cerrar-vista:hover {
    background: var(--m3c-surface-container-highest);
    color: var(--m3c-on-surface);
  }

  .mashup {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-4);
    padding: var(--ns-espacio-5);
    border-radius: var(--ns-radio-xl);
    background: linear-gradient(
      180deg,
      color-mix(in srgb, var(--m3c-tertiary-container) 40%, var(--m3c-surface-container)) 0%,
      var(--m3c-surface-container-low) 22rem
    );
  }

  .cabecera {
    display: flex;
    align-items: flex-end;
    gap: var(--ns-espacio-5);
  }

  .portada {
    display: grid;
    flex: 0 0 10rem;
    place-items: center;
    aspect-ratio: 1;
    border-radius: var(--ns-radio-md);
    background: linear-gradient(
      135deg,
      var(--m3c-tertiary-container),
      var(--m3c-primary-container)
    );
    color: var(--m3c-on-tertiary-container);
    box-shadow: 0 12px 28px var(--m3c-shadow);
  }

  .datos {
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-2);
    min-width: 0;
  }

  .estado {
    color: var(--m3c-tertiary);
    font-size: var(--ns-texto-xs);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  h2 {
    margin: 0;
    font-size: clamp(var(--ns-texto-xl), 3vw, var(--ns-texto-2xl));
    line-height: 1.1;
    overflow-wrap: anywhere;
  }

  h3 {
    margin: 0;
    font-size: var(--ns-texto-md);
  }

  p {
    margin: 0;
  }

  .cifras {
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-sm);
  }

  .progreso {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--ns-espacio-2) var(--ns-espacio-4);
    padding: var(--ns-espacio-4);
    border: 1px solid var(--m3c-outline-variant);
    border-radius: var(--ns-radio-lg);
    background: var(--m3c-surface-container);
  }

  .linea-progreso {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--ns-espacio-2);
    font-size: var(--ns-texto-sm);
  }

  .unidades,
  .pct {
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-xs);
    font-variant-numeric: tabular-nums;
  }

  .pct {
    margin-left: auto;
  }

  .pista-barra {
    grid-column: 1;
    height: 0.375rem;
    overflow: hidden;
    border-radius: var(--ns-radio-pleno);
    background: var(--m3c-surface-container-highest);
  }

  .relleno {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--m3c-tertiary);
    transition: width var(--ns-duracion-media) var(--ns-curva);
  }

  .progreso .pildora {
    grid-column: 2;
    grid-row: 1 / span 2;
  }

  .problema {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ns-espacio-3);
    padding: var(--ns-espacio-3) var(--ns-espacio-4);
    border-radius: var(--ns-radio-lg);
    background: var(--m3c-surface-container-high);
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-sm);
  }

  .problema p {
    flex: 1 1 14rem;
  }

  .acciones {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ns-espacio-3);
  }

  .reproducir {
    display: grid;
    place-items: center;
    width: 3.5rem;
    height: 3.5rem;
    padding: 0;
    border: none;
    border-radius: var(--ns-radio-pleno);
    background: var(--m3c-primary);
    color: var(--m3c-on-primary);
    box-shadow: 0 6px 18px var(--m3c-shadow);
    cursor: pointer;
    transition: transform var(--ns-duracion-corta) var(--ns-curva);
  }

  .reproducir:hover {
    transform: scale(1.05);
  }

  .pildora {
    display: inline-flex;
    align-items: center;
    gap: var(--ns-espacio-2);
    height: 2.5rem;
    padding: 0 var(--ns-espacio-4);
    border: 1px solid var(--m3c-outline-variant);
    border-radius: var(--ns-radio-pleno);
    background: none;
    color: var(--m3c-on-surface);
    font: inherit;
    font-size: var(--ns-texto-sm);
    font-weight: 600;
    cursor: pointer;
  }

  .pildora:hover {
    background: var(--m3c-surface-container-highest);
  }

  /* Borrar va al final y sin borde: no compite con escuchar ni exportar. */
  .pildora.discreta {
    margin-left: auto;
    border-color: transparent;
    color: var(--m3c-on-surface-variant);
  }

  .cedido {
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-1);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .cedido li {
    display: flex;
    align-items: center;
    gap: var(--ns-espacio-2);
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-sm);
  }

  /* La línea de tiempo: un carril por papel, un bloque por fragmento. */
  .linea {
    --ancho-nombre: 4.5rem;
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-1);
  }

  .carril {
    display: grid;
    grid-template-columns: var(--ancho-nombre) minmax(0, 1fr);
    align-items: center;
  }

  .nombre-carril {
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-xs);
    font-weight: 600;
  }

  .pista-carril {
    position: relative;
    height: 2.25rem;
    border-radius: var(--ns-radio-sm);
    background: var(--m3c-surface-container);
  }

  .bloque {
    position: absolute;
    top: 0.125rem;
    bottom: 0.125rem;
    min-width: 0.25rem;
    padding: 0 var(--ns-espacio-1);
    overflow: hidden;
    border: 1px solid color-mix(in srgb, var(--m3c-surface) 60%, transparent);
    border-radius: var(--ns-radio-sm);
    font: inherit;
    font-size: var(--ns-texto-xs);
    text-align: left;
    white-space: nowrap;
    text-overflow: ellipsis;
    cursor: pointer;
  }

  .bloque span {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .bloque:hover {
    filter: brightness(1.08);
  }

  .bloque:focus-visible {
    z-index: 1;
    outline: 2px solid var(--m3c-primary);
    outline-offset: 1px;
  }

  .c0 {
    background: var(--m3c-primary-container);
    color: var(--m3c-on-primary-container);
  }

  .c1 {
    background: var(--m3c-tertiary-container);
    color: var(--m3c-on-tertiary-container);
  }

  .c2 {
    background: var(--m3c-secondary-container);
    color: var(--m3c-on-secondary-container);
  }

  .c3 {
    background: var(--m3c-primary);
    color: var(--m3c-on-primary);
  }

  .c4 {
    background: var(--m3c-tertiary);
    color: var(--m3c-on-tertiary);
  }

  .c5 {
    background: var(--m3c-secondary);
    color: var(--m3c-on-secondary);
  }

  .cabezal {
    position: absolute;
    top: 0;
    bottom: 1.25rem;
    width: 2px;
    border-radius: 1px;
    background: var(--m3c-on-surface);
    pointer-events: none;
  }

  .eje {
    display: flex;
    justify-content: space-between;
    margin-left: var(--ancho-nombre);
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-xs);
    font-variant-numeric: tabular-nums;
  }

  .pistas {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .pista {
    display: grid;
    grid-template-columns: 0.75rem 2.5rem minmax(0, 1fr) 3.25rem;
    align-items: center;
    gap: var(--ns-espacio-3);
    width: 100%;
    min-height: 3.5rem;
    padding: 0 var(--ns-espacio-3);
    border: none;
    border-radius: var(--ns-radio-sm);
    background: none;
    color: var(--m3c-on-surface);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .pista:hover:not(:disabled) {
    background: var(--m3c-surface-container);
  }

  .pista:disabled {
    cursor: default;
  }

  .punto {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: var(--ns-radio-pleno);
  }

  .mini {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: var(--ns-radio-sm);
    background: var(--m3c-surface-container-highest);
    object-fit: cover;
  }

  .datos-pista {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .datos-pista strong,
  .datos-pista small {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .datos-pista small {
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-xs);
  }

  .duracion {
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-xs);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  @container carcasa (max-width: 40rem) {
    .cabecera {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
