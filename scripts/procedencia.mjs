#!/usr/bin/env node
/**
 * Monta la lista de huellas de una publicación y la firma.
 *
 * Uso: node scripts/procedencia.mjs <carpeta> [--clave <par-de-claves.pkcs8>]
 *
 * Escribe `SHA256SUMS` con la huella de cada archivo de la carpeta y, con la clave,
 * `SHA256SUMS.sig`: la firma Ed25519 de NB Digital. Después comprueba la firma
 * contra la clave pública **del repositorio** (docs/architecture/api/), no contra la
 * que acaba de usar: firmar con una clave equivocada daría una firma válida que
 * nadie podría comprobar.
 *
 * Quedan fuera `launcher-catalog.json` y su firma: el catálogo de NB Launcher caduca
 * y se renueva cada semana sin tocar los paquetes, y lleva su propia firma minisign.
 * Con él dentro, la lista dejaría de cuadrar en la primera renovación.
 *
 * La firma se hace con `openssl`, que es lo mismo con lo que se verifica
 * (docs/developer/releases.md): no hay una segunda implementación que pueda divergir.
 */

import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

const ROOT = resolve(import.meta.dirname, '..');
const CLAVE_PUBLICA = join(ROOT, 'docs', 'architecture', 'api', 'clave-de-publicacion.pem');
const FUERA = new Set(['SHA256SUMS', 'SHA256SUMS.sig', 'launcher-catalog.json', 'launcher-catalog.json.sig']);

const carpeta = process.argv[2];
if (!carpeta) {
  console.error('Uso: node scripts/procedencia.mjs <carpeta> [--clave <archivo.pkcs8>]');
  process.exit(2);
}
const i = process.argv.indexOf('--clave');
const clave = i > 0 ? process.argv[i + 1] : null;

const archivos = readdirSync(carpeta)
  .filter((n) => !FUERA.has(n) && statSync(join(carpeta, n)).isFile())
  .sort();
const lineas = archivos.map(
  (n) => `${createHash('sha256').update(readFileSync(join(carpeta, n))).digest('hex')}  ${n}`
);
const sumas = join(carpeta, 'SHA256SUMS');
writeFileSync(sumas, `${lineas.join('\n')}\n`);
console.log(`SHA256SUMS: ${archivos.length} archivos.`);

if (!clave) {
  console.log('Sin --clave: la lista queda sin firmar.');
  process.exit(0);
}
const firma = join(carpeta, 'SHA256SUMS.sig');
execFileSync('openssl', ['pkeyutl', '-sign', '-inkey', clave, '-rawin', '-in', sumas, '-out', firma]);
execFileSync('openssl', [
  'pkeyutl',
  '-verify',
  '-pubin',
  '-inkey',
  CLAVE_PUBLICA,
  '-rawin',
  '-in',
  sumas,
  '-sigfile',
  firma
]);
console.log('SHA256SUMS.sig: firmada y comprobada contra la clave pública del repositorio.');
