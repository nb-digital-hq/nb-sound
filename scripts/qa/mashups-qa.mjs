#!/usr/bin/env node
// Pruebas de DJ Mashups sobre la copia aislada, con la aplicación
// real, el modelo de karaoke real y la música de la copia. Lo orquesta
// scripts/qa/ui-qa.sh mashups; cada escenario deja una línea QA_MASHUP_*.

import { execFileSync } from 'node:child_process';

import { Inspector, esperar } from './inspector.mjs';

const ESCENARIO = process.argv[2];
const DB = process.env.NB_QA_DB;
const inspector = await Inspector.conectar();

function sql(consulta) {
  return execFileSync('sqlite3', ['-readonly', '-separator', '\t', DB, consulta], {
    encoding: 'utf8'
  }).trim();
}

const invocar = (comando, argumentos = {}) =>
  inspector.evaluar(
    `window.__TAURI_INTERNALS__.invoke(${JSON.stringify(comando)}, ${JSON.stringify(argumentos)})`
  );
const estado = () => invocar('reproductor_estado');
const dom = (expresion) => inspector.evaluar(expresion);

async function esperarHasta(prueba, limite, descripcion, cada = 200) {
  const hasta = Date.now() + limite;
  let ultimo;
  while (Date.now() < hasta) {
    try {
      ultimo = await prueba();
      if (ultimo) return ultimo;
    } catch (error) {
      ultimo = String(error);
    }
    await esperar(cada);
  }
  throw new Error(`No se cumplió: ${descripcion}. Último: ${JSON.stringify(ultimo)}`);
}

async function irAlDj() {
  // Por el enlace de la barra lateral, como el usuario: la ruta cargada a mano
  // no existe como archivo en el paquete.
  await esperarHasta(
    () => dom(`document.querySelector('a[href="/dj"]') ? true : null`),
    30000,
    'la barra lateral'
  ).catch(async (e) => {
    const donde = await dom(`location.href + ' | ' + [...document.querySelectorAll('a')].map((a) => a.getAttribute('href')).join(',') + ' | ' + document.body.innerText.slice(0, 300)`);
    throw new Error(`${e.message} en ${donde}`);
  });
  await dom(`(() => { document.querySelector('a[href="/dj"]').click(); return true; })()`);
  await esperarHasta(
    () => dom(`document.querySelector('.alternar') ? true : null`),
    60000,
    'la vista del DJ'
  ).catch(async (e) => {
    const donde = await dom(`location.href + ' | ' + document.body.innerText.slice(0, 400)`);
    throw new Error(`${e.message} en ${donde}`);
  });
}

/** Lo que se ve: el título, cuántas experiencias visibles y el historial. */
const vista = () =>
  dom(`(() => ({
    titulo: document.querySelector('.vista h1')?.textContent?.trim(),
    boton: document.querySelector('.alternar')?.getAttribute('aria-label'),
    visibles: [...document.querySelectorAll('.experiencia')].filter((e) => !e.hidden).length,
    formularioPrivado: Boolean(document.querySelector('.experiencia:not([hidden]) textarea')),
    formularioMashups: Boolean(document.querySelector('.experiencia:not([hidden]) .estilos')),
    historiales: document.querySelectorAll('.historial').length,
    marca: document.querySelector('.historial')?.dataset?.qa ?? null
  }))()`);

const pulsar = (selector) =>
  dom(`(() => { const b = document.querySelector(${JSON.stringify(selector)}); b?.click(); return Boolean(b); })()`);

async function alternar() {
  await irAlDj();
  // Se empieza siempre por el DJ Privado.
  let v = await vista();
  if (v.titulo !== 'DJ Privado') {
    await pulsar('.alternar');
    v = await esperarHasta(async () => ((await vista()).titulo === 'DJ Privado' ? vista() : null), 5000, 'DJ Privado');
  }
  if (v.visibles !== 1 || !v.formularioPrivado || v.historiales !== 1) {
    throw new Error(`DJ Privado mal montado: ${JSON.stringify(v)}`);
  }
  // Un borrador en el DJ Privado y una marca en el historial: al volver, los dos
  // tienen que ser los mismos (el historial no se desmonta).
  await dom(`(() => {
    const t = document.querySelector('.experiencia:not([hidden]) textarea');
    t.value = 'borrador de prueba'; t.dispatchEvent(new Event('input', { bubbles: true }));
    document.querySelector('.historial').dataset.qa = 'mismo';
    return true;
  })()`);
  const t0 = Date.now();
  await pulsar('.alternar');
  const m = await esperarHasta(
    async () => {
      const x = await vista();
      return x.titulo === 'DJ Mashups' && x.formularioMashups ? x : null;
    },
    5000,
    'DJ Mashups'
  );
  const cambioMs = Date.now() - t0;
  if (m.visibles !== 1 || m.formularioPrivado || m.historiales !== 1 || m.marca !== 'mismo') {
    throw new Error(`DJ Mashups mal montado: ${JSON.stringify(m)}`);
  }
  if (m.boton !== 'Cambiar a DJ Privado') throw new Error(`El botón no dice adónde lleva: ${m.boton}`);
  await pulsar('.alternar');
  await esperarHasta(async () => ((await vista()).titulo === 'DJ Privado' ? true : null), 5000, 'vuelta');
  const borrador = await dom(`document.querySelector('.experiencia:not([hidden]) textarea')?.value`);
  const marca = (await vista()).marca;
  if (borrador !== 'borrador de prueba' || marca !== 'mismo') {
    throw new Error(`Se perdió el estado al alternar: ${JSON.stringify({ borrador, marca })}`);
  }
  // El botón es un botón: se alcanza con el tabulador y Enter lo activa.
  const enfocable = await dom(
    `(() => { const b = document.querySelector('.alternar'); b.focus(); return document.activeElement === b && b.tagName === 'BUTTON'; })()`
  );
  if (!enfocable) throw new Error('El botón de alternar no es enfocable');
  console.log('QA_MASHUP_ALTERNAR', JSON.stringify({ cambioMs, borrador: true, historialUnico: true }));
}

async function generar() {
  await irAlDj();
  if ((await vista()).titulo !== 'DJ Mashups') await pulsar('.alternar');
  const inicial = await invocar('mashup_estado');
  if (!inicial.conStems) throw new Error('La copia no tiene el agregado de karaoke enlazado');
  // Una canción de la biblioteca sonando: la generación no puede cortarla.
  const pista = Number(sql("SELECT id FROM pistas WHERE estado='biblioteca' AND duracion_seg > 400 ORDER BY id LIMIT 1;") || sql("SELECT id FROM pistas WHERE estado='biblioteca' AND duracion_seg > 200 ORDER BY id LIMIT 1;"));
  await invocar('reproductor_orden', { orden: { tipo: 'reproducirPistas', ids: [pista], desde: 0 } });
  // El formulario: 4 canciones, 3 minutos, equilibrado.
  await dom(`(() => {
    const fichas = [...document.querySelectorAll('.experiencia:not([hidden]) .ficha')];
    fichas.find((f) => f.textContent.trim() === '4')?.click();
    fichas.find((f) => f.textContent.trim() === '3 min')?.click();
    return true;
  })()`);
  const t0 = Date.now();
  await pulsar('.experiencia:not([hidden]) .boton');
  const id = await esperarHasta(
    () => Number(sql("SELECT id FROM mashups WHERE estado='generando' ORDER BY id DESC LIMIT 1;")) || null,
    10000,
    'el mashup creado'
  );
  // Mientras genera: la barra de progreso real y la reproducción avanzando.
  const muestras = [];
  let posAnterior = (await estado()).posicionSeg;
  const latencias = [];
  while (true) {
    const e = sql(`SELECT estado FROM mashups WHERE id=${id};`);
    if (e !== 'generando') break;
    const t = Date.now();
    const s = await estado();
    latencias.push(Date.now() - t);
    const barra = await dom(
      `document.querySelector('[role=progressbar]')?.getAttribute('aria-valuenow') ?? null`
    );
    muestras.push({ s: (Date.now() - t0) / 1000, pct: Number(barra), pos: s.posicionSeg, estado: s.estado });
    if (Date.now() - t0 > 3600000) throw new Error('La generación tardó más de una hora');
    await esperar(2000);
  }
  const total = (Date.now() - t0) / 1000;
  const final = sql(`SELECT estado, coalesce(error,''), duracion_seg, metricas FROM mashups WHERE id=${id};`).split('\t');
  if (final[0] !== 'lista') throw new Error(`El mashup acabó en ${final[0]} ${final[1]}`);
  const avanzando = muestras.filter((m) => m.estado === 'reproduciendo');
  const posiciones = avanzando.map((m) => m.pos);
  const progresoMonotono = muestras.every((m, i) => i === 0 || m.pct >= muestras[i - 1].pct || Number.isNaN(m.pct));
  if (avanzando.length > 2 && posiciones.at(-1) <= posiciones[0]) {
    throw new Error('La reproducción se paró mientras se generaba');
  }
  // El historial muestra el mashup con su tipo.
  await esperarHasta(
    () => dom(`[...document.querySelectorAll('.historial li')].some((li) => li.textContent.includes('Mashup'))`),
    10000,
    'el mashup en el historial'
  );
  // Escucharlo desde la vista: la barra dice DJ Mashups y la cola espera.
  await pulsar('.experiencia:not([hidden]) .reproducir');
  const sonando = await esperarHasta(
    async () => {
      const e = await estado();
      return e.mashupId === id && e.estado === 'reproduciendo' ? e : null;
    },
    10000,
    'el mashup sonando'
  );
  const barra = await dom(`document.querySelector('.reproductor .titulo')?.textContent?.trim()`);
  await esperar(3000);
  const avanza = (await estado()).posicionSeg > 1;
  // Un bloque de la línea de tiempo lleva a su momento.
  const bloques = await dom(`document.querySelectorAll('.bloque').length`);
  await pulsar('.bloque:last-of-type');
  await esperar(500);
  const tras = await estado();
  // Dejar de tocarlo devuelve la cola del usuario.
  await invocar('reproductor_orden', { orden: { tipo: 'reproducirPista', id: pista } });
  const vuelta = await esperarHasta(async () => ((await estado()).mashupId == null ? estado() : null), 5000, 'la cola de vuelta');
  console.log(
    'QA_MASHUP_GENERAR',
    JSON.stringify({
      id,
      segundos: total,
      latenciaMaxMs: Math.max(...latencias),
      latenciaMediaMs: latencias.reduce((a, b) => a + b, 0) / Math.max(1, latencias.length),
      muestras: muestras.length,
      progresoMonotono,
      reproduccionDurante: posiciones.length > 1 ? posiciones.at(-1) - posiciones[0] : null,
      duracion: Number(final[2]),
      metricas: JSON.parse(final[3]),
      barra,
      avanza,
      bloques,
      saltoA: tras.posicionSeg,
      colaDeVuelta: vuelta.pistaId === pista,
      sonandoDuracion: sonando.duracionSeg
    })
  );
}

async function cancelar() {
  await irAlDj();
  if ((await vista()).titulo !== 'DJ Mashups') await pulsar('.alternar');
  const id = await invocar('mashup_generar', {
    peticion: { duracionSeg: 600, canciones: 32, estilo: 'relampago', semilla: 77, pistas: [], playlistId: null }
  });
  await esperar(4000);
  await invocar('mashup_cancelar', { mashupId: id });
  const t0 = Date.now();
  await esperarHasta(() => sql(`SELECT estado FROM mashups WHERE id=${id};`) === 'cancelada', 600000, 'cancelado', 250);
  const ms = Date.now() - t0;
  const libre = (await invocar('mashup_estado')).enCurso === null;
  console.log('QA_MASHUP_CANCELAR', JSON.stringify({ id, msHastaCancelado: ms, libre }));
}

async function reabierta() {
  await irAlDj();
  const ultimo = Number(sql("SELECT id FROM mashups WHERE estado='lista' ORDER BY id DESC LIMIT 1;"));
  const filas = await invocar('dj_historial_completo', { texto: '' });
  const tipos = [...new Set(filas.map((f) => f.tipo))];
  const esta = filas.some((f) => f.tipo === 'mashup' && f.id === ultimo);
  await invocar('mashup_reproducir', { mashupId: ultimo, desdeSeg: 0 });
  const e = await esperarHasta(async () => ((await estado()).mashupId === ultimo ? estado() : null), 5000, 'suena tras reabrir');
  await invocar('reproductor_orden', { orden: { tipo: 'pausarReanudar' } });
  console.log('QA_MASHUP_REABIERTA', JSON.stringify({ ultimo, esta, tipos, suena: e.estado }));
  if (!esta) throw new Error('El mashup no sobrevivió al cierre');
}

try {
  if (ESCENARIO === 'alternar') await alternar();
  else if (ESCENARIO === 'generar') await generar();
  else if (ESCENARIO === 'cancelar') await cancelar();
  else if (ESCENARIO === 'reabierta') await reabierta();
  else throw new Error(`Escenario desconocido: ${ESCENARIO}`);
} finally {
  inspector.cerrar();
}
