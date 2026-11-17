/**
 * Textos en portugués de Brasil de A ciegas.
 */
import type { Indicio, Modo } from '#lib/bindings/index.ts';
import type { TextosACiegas } from './es-aciegas';
import type { Forma } from './forma';

export const ptACiegas: Forma<TextosACiegas> = {
  titulo: 'Às Cegas',
  sinMusica: 'Para jogar é preciso ter música: adicione músicas à sua biblioteca e volte.',
  subtitulo: 'Redescubra sua música: adivinhe músicas pela capa ou por alguns segundos de som.',
  modos: {
    portada: { nombre: 'Pela capa', detalle: 'Você vê a capa e adivinha a música.' },
    audio: {
      nombre: 'Pelo som',
      detalle: 'Você ouve alguns segundos sem saber o que está tocando.'
    },
    redescubrimiento: {
      nombre: 'Redescobrir',
      detalle: 'Músicas de que você gostou e que não tocam há tempos.'
    },
    nunca_eliges: {
      nombre: 'A que você nunca escolhe',
      detalle: 'Músicas da sua biblioteca que você quase nunca põe.'
    }
  } satisfies Record<Modo, { nombre: string; detalle: string }>,
  canciones: (n: number) => (n === 1 ? '1 música' : `${n.toLocaleString('pt-BR')} músicas`),
  retos: 'Desafios por rodada',
  empezar: 'Começar',
  cargando: 'Preparando a partida…',
  sinCanciones: 'Ainda não há músicas para este modo.',
  errorAlIniciar: 'Não foi possível iniciar a partida. Tente novamente.',
  reto: (n: number, total: number) => `Desafio ${n} de ${total}`,
  marcador: (acertados: number) => (acertados === 1 ? '1 acerto' : `${acertados} acertos`),
  oculto: '???',
  campos: { titulo: 'Título', artista: 'Artista', album: 'Álbum' },
  escuchar: 'Ouvir o trecho',
  segundos: (s: number) => `${s} s`,
  duracionFragmento: 'Duração do trecho',
  escribir: 'Como se chama?',
  comprobar: 'Verificar',
  laSe: 'Eu sei!',
  noSeEscribe: 'Este título não é escrito com letras latinas: se você reconhecer, diga.',
  acierto: 'Correto!',
  cerca: 'Quase…',
  fallo: 'Não é essa.',
  fallos: (n: number) => (n === 1 ? '1 tentativa errada' : `${n} tentativas erradas`),
  verArtista: 'Ver o artista',
  verAlbum: 'Ver o álbum',
  rendirse: 'Desistir',
  siguiente: 'Próximo',
  anterior: 'Anterior',
  terminar: 'Ver o resumo',
  indicios: 'Dicas',
  nombresIndicio: {
    alfabeto: 'Alfabeto',
    empiezaCon: 'Começa com',
    terminaCon: 'Termina em',
    cantidadPalabras: 'Palavras',
    cantidadLetras: 'Letras'
  } satisfies Record<Indicio, string>,
  alfabetos: {
    latino: 'latino',
    griego: 'grego',
    cirilico: 'cirílico',
    hebreo: 'hebraico',
    arabe: 'árabe',
    devanagari: 'devanágari',
    japones: 'japonês',
    chino: 'chinês',
    coreano: 'coreano',
    otro: 'outro'
  } as Record<string, string>,
  escucharEntera: 'Ouvir inteira',
  aLaCola: 'Adicionar à fila',
  enCola: 'Na fila.',
  resumen: {
    titulo: 'Fim da rodada',
    cifras: (acertados: number, revelados: number, pasados: number) =>
      [
        `${acertados} ${acertados === 1 ? 'acertada' : 'acertadas'}`,
        `${revelados} ${revelados === 1 ? 'revelada' : 'reveladas'}`,
        `${pasados} ${pasados === 1 ? 'pulada' : 'puladas'}`
      ].join(' · '),
    otra: 'Outra rodada',
    cambiar: 'Mudar de modo',
    estados: {
      acertado: 'Acertada',
      revelado: 'Revelada',
      pasado: 'Pulada',
      en_curso: ''
    } as Record<string, string>,
    escuchar: (titulo: string) => `Ouvir ${titulo}`
  }
};
