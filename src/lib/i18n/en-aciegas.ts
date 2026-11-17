/**
 * Textos en inglés de A ciegas.
 */
import type { Indicio, Modo } from '#lib/bindings/index.ts';
import type { TextosACiegas } from './es-aciegas';
import type { Forma } from './forma';

export const enACiegas: Forma<TextosACiegas> = {
  titulo: 'Blind Test',
  sinMusica: 'You need music to play: add songs to your library and come back.',
  subtitulo: 'Rediscover your music: guess songs by their cover or by a few seconds of sound.',
  modos: {
    portada: { nombre: 'By the cover', detalle: 'You see the cover and guess the song.' },
    audio: {
      nombre: 'By the sound',
      detalle: 'You hear a few seconds without knowing what is playing.'
    },
    redescubrimiento: {
      nombre: 'Rediscover',
      detalle: 'Songs you liked that have not played in a while.'
    },
    nunca_eliges: {
      nombre: 'The one you never pick',
      detalle: 'Songs in your library you almost never play.'
    }
  } satisfies Record<Modo, { nombre: string; detalle: string }>,
  canciones: (n: number) => (n === 1 ? '1 song' : `${n.toLocaleString('en')} songs`),
  retos: 'Challenges per round',
  empezar: 'Start',
  cargando: 'Preparing the game…',
  sinCanciones: 'There are no songs for this mode yet.',
  errorAlIniciar: 'The game could not be started. Try again.',
  reto: (n: number, total: number) => `Challenge ${n} of ${total}`,
  marcador: (acertados: number) => `${acertados} correct`,
  oculto: '???',
  campos: { titulo: 'Title', artista: 'Artist', album: 'Album' },
  escuchar: 'Listen to the clip',
  segundos: (s: number) => `${s} s`,
  duracionFragmento: 'Clip length',
  escribir: 'What is it called?',
  comprobar: 'Check',
  laSe: 'I know it!',
  noSeEscribe: 'This title is not written in Latin letters: if you recognize it, say so.',
  acierto: 'Correct!',
  cerca: 'Almost…',
  fallo: 'That is not it.',
  fallos: (n: number) => (n === 1 ? '1 failed attempt' : `${n} failed attempts`),
  verArtista: 'Show the artist',
  verAlbum: 'Show the album',
  rendirse: 'Give up',
  siguiente: 'Next',
  anterior: 'Previous',
  terminar: 'See the summary',
  indicios: 'Hints',
  nombresIndicio: {
    alfabeto: 'Alphabet',
    empiezaCon: 'Starts with',
    terminaCon: 'Ends with',
    cantidadPalabras: 'Words',
    cantidadLetras: 'Letters'
  } satisfies Record<Indicio, string>,
  alfabetos: {
    latino: 'Latin',
    griego: 'Greek',
    cirilico: 'Cyrillic',
    hebreo: 'Hebrew',
    arabe: 'Arabic',
    devanagari: 'Devanagari',
    japones: 'Japanese',
    chino: 'Chinese',
    coreano: 'Korean',
    otro: 'other'
  } as Record<string, string>,
  escucharEntera: 'Listen to the whole song',
  aLaCola: 'Add to queue',
  enCola: 'In the queue.',
  resumen: {
    titulo: 'End of the round',
    cifras: (acertados: number, revelados: number, pasados: number) =>
      [`${acertados} correct`, `${revelados} revealed`, `${pasados} skipped`].join(' · '),
    otra: 'Another round',
    cambiar: 'Change mode',
    estados: {
      acertado: 'Correct',
      revelado: 'Revealed',
      pasado: 'Skipped',
      en_curso: ''
    } as Record<string, string>,
    escuchar: (titulo: string) => `Listen to ${titulo}`
  }
};
