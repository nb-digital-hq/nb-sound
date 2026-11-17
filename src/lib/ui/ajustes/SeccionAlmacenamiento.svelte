<!--
  @component
  Ajustes > Almacenamiento: lo que NB Sound guarda en este equipo, por
  categoría. Cada una dice cuánto ocupa y, si se puede estimar, cuánto ocuparía
  con toda la biblioteca, medido de lo que ya hay. Las separaciones tienen su
  propio límite; lo que se puede rehacer se puede vaciar, diciendo antes cuánto
  costaría rehacerlo. Lo que no se puede rehacer solo se muestra.
-->
<script lang="ts">
  import {
    almacenamientoFijarTope,
    almacenamientoInforme,
    almacenamientoLiberar,
    type CategoriaAlmacen,
    type InformeAlmacen
  } from '#lib/core/ipc/almacenamiento.ts';
  import { notificar } from '#lib/core/notificaciones.svelte.ts';
  import { textosAparte } from '#lib/i18n/aparte.ts';
  import { localeActual } from '#lib/i18n/index.ts';
  import DialogoConfirmar from '#lib/ui/DialogoConfirmar.svelte';
  import MenuSelector from '#lib/ui/MenuSelector.svelte';

  const t = textosAparte.almacenamiento;
  const GB = 1024 ** 3;
  const TOPES = [2, 5, 10, 20, 50, 100, 200].map((g) => g * GB);

  let informe = $state<InformeAlmacen | null>(null);
  let porLiberar = $state<CategoriaAlmacen | null>(null);
  let ocupado = $state(false);

  async function leer() {
    informe = await almacenamientoInforme().catch(() => null);
  }
  void leer();

  function tam(b: number): string {
    const f = (v: number, u: string) =>
      `${v.toLocaleString(localeActual(), { maximumFractionDigits: v < 10 ? 1 : 0 })} ${u}`;
    if (b >= GB) return f(b / GB, 'GB');
    if (b >= 1024 ** 2) return f(b / 1024 ** 2, 'MB');
    return f(Math.max(b, 0) / 1024, 'KB');
  }

  function duracion(seg: number): string {
    const h = seg / 3600;
    if (h >= 1) return `${h.toLocaleString(localeActual(), { maximumFractionDigits: 1 })} h`;
    return `${Math.max(1, Math.round(seg / 60))} min`;
  }

  const total = $derived(informe?.categorias.reduce((s, c) => s + c.bytes, 0) ?? 0);
  // Lo que podría llegar a ocupar: lo limitado hasta su límite, lo demás como
  // está. Si no cabe en lo libre, se avisa.
  const exceso = $derived.by(() => {
    if (!informe || informe.libre === null) return 0;
    const potencial = informe.categorias.reduce((s, c) => s + Math.max(c.bytes, c.tope ?? 0), 0);
    return potencial - (total + informe.libre);
  });

  const opcionesTope = (actual: number) => {
    const valores = TOPES.includes(actual) ? TOPES : [...TOPES, actual].sort((a, b) => a - b);
    return valores.map((v) => ({ valor: v, etiqueta: tam(v) }));
  };

  async function fijar(bytes: number) {
    try {
      await almacenamientoFijarTope(bytes);
    } catch {
      notificar.error(textosAparte.comun.error);
    }
    await leer();
  }

  async function liberar() {
    if (!porLiberar) return;
    ocupado = true;
    try {
      const n = await almacenamientoLiberar(porLiberar.id);
      notificar.exito(t.liberado(tam(n)));
    } catch {
      notificar.error(textosAparte.comun.error);
    } finally {
      ocupado = false;
      porLiberar = null;
      await leer();
    }
  }
</script>

<section id="almacenamiento" aria-labelledby="t-almacenamiento">
  <h2 id="t-almacenamiento">{t.titulo}</h2>
  <p class="nota">{t.subtitulo}</p>

  {#if informe}
    <p class="resumen">
      <strong>{t.total(tam(total))}</strong>
      {#if informe.libre !== null}<span>{t.libre(tam(informe.libre))}</span>{/if}
    </p>
    {#if exceso > 0}
      <p class="aviso" role="status">{t.noCabe(tam(exceso))}</p>
    {/if}

    <ul class="categorias">
      {#each informe.categorias as c (c.id)}
        <li>
          <div class="cabeza">
            <span class="nombre">{t.categorias[c.id]}</span>
            <span class="cifra">{tam(c.bytes)}</span>
          </div>
          <p class="nota">{t.ayudas[c.id]}</p>
          {#if c.todaLaBiblioteca !== null}
            <p class="nota">{t.todaLaBiblioteca(tam(c.todaLaBiblioteca))}</p>
          {/if}
          {#if c.tope !== null}
            <div class="barra" aria-hidden="true">
              <span style:width={`${Math.min(100, (c.bytes / c.tope) * 100)}%`}></span>
            </div>
          {/if}
          {#if c.tope !== null || c.liberable}
            <div class="acciones">
              {#if c.tope !== null}
                <span class="rotulo">{t.limite}</span>
                <MenuSelector
                  opciones={opcionesTope(c.tope)}
                  valor={c.tope}
                  controlado
                  compacto
                  ariaLabel={`${t.limite}: ${t.categorias[c.id]}`}
                  onchange={(v) => void fijar(v)}
                />
              {/if}
              {#if c.liberable}
                <button
                  type="button"
                  class="boton"
                  disabled={c.bytes === 0}
                  onclick={() => (porLiberar = c)}
                >
                  {t.liberar}
                </button>
              {/if}
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>

<DialogoConfirmar
  abierto={porLiberar !== null}
  titulo={t.liberar}
  mensaje={porLiberar
    ? `${t.confirmarLiberar(t.categorias[porLiberar.id].toLowerCase(), tam(porLiberar.bytes))} ${
        porLiberar.rehacerSeg !== null ? t.rehacer(duracion(porLiberar.rehacerSeg)) : t.rehacerSolo
      }`
    : ''}
  textoConfirmar={t.liberar}
  destructivo
  {ocupado}
  onconfirmar={() => void liberar()}
  oncerrar={() => (porLiberar = null)}
/>

<style>
  .nota {
    margin: 0;
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-sm);
  }

  .resumen {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ns-espacio-2) var(--ns-espacio-4);
    margin: var(--ns-espacio-3) 0 0;
    font-size: var(--ns-texto-sm);
  }

  .resumen span {
    color: var(--m3c-on-surface-variant);
  }

  .aviso {
    margin: var(--ns-espacio-2) 0 0;
    color: var(--m3c-error);
    font-size: var(--ns-texto-sm);
  }

  .categorias {
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-3);
    margin: var(--ns-espacio-4) 0 0;
    padding: 0;
    list-style: none;
  }

  .categorias li {
    display: flex;
    flex-direction: column;
    gap: var(--ns-espacio-2);
    padding: var(--ns-espacio-4);
    border-radius: var(--ns-radio-lg);
    background: var(--m3c-surface-container-high);
  }

  .cabeza {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--ns-espacio-3);
  }

  .nombre {
    font-weight: 600;
  }

  .cifra {
    font-variant-numeric: tabular-nums;
    font-weight: 700;
  }

  .barra {
    height: 0.375rem;
    overflow: hidden;
    border-radius: var(--ns-radio-pleno);
    background: var(--m3c-surface-container-highest);
  }

  .barra span {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--m3c-primary);
  }

  .acciones {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ns-espacio-2) var(--ns-espacio-3);
  }

  .rotulo {
    color: var(--m3c-on-surface-variant);
    font-size: var(--ns-texto-sm);
  }

  .boton {
    height: 2.25rem;
    margin-left: auto;
    padding: 0 var(--ns-espacio-4);
    border: 1px solid var(--m3c-outline-variant);
    border-radius: var(--ns-radio-pleno);
    background: transparent;
    color: var(--m3c-on-surface);
    font: inherit;
    font-size: var(--ns-texto-sm);
    font-weight: 600;
    cursor: pointer;
  }

  .boton:hover:not(:disabled) {
    background: var(--m3c-surface-container-highest);
  }

  .boton:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
