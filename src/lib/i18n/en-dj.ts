/**
 * Textos en inglés del DJ Privado y de DJ Mashups.
 */
import type { Tecnica } from '#lib/bindings/index.ts';
import type { TextosDj } from './es-dj';
import type { Forma } from './forma';

export const enDj: Forma<TextosDj> = {
  titulo: 'Private DJ',
  subtitulo:
    'Tell it what you feel like and it builds a mixed session from your music, like a DJ would.',
  cerrar: 'Close (stays in the history)',
  modos: {
    aMashups: 'DJ Mashups',
    aPrivado: 'Private DJ',
    irAMashups: 'Switch to DJ Mashups',
    irAPrivado: 'Switch to Private DJ'
  },
  mashups: {
    titulo: 'DJ Mashups',
    subtitulo:
      'Composes a new piece from fragments of your songs: vocals from some over the beats of others, at the same tempo and in keys that fit.',
    origen: 'Songs',
    toda: 'Your whole library',
    playlist: (nombre: string) => `Playlist: ${nombre}`,
    cuantas: 'How many',
    duracion: 'Length',
    minutos: (n: number) => `${n} min`,
    estilo: 'Style',
    estilos: {
      equilibrado: {
        nombre: 'Balanced',
        ayuda: 'Eight-bar phrases and an arc-shaped build.'
      },
      fiesta: { nombre: 'Party', ayuda: 'More layers and energy that grows until the end.' },
      suave: { nombre: 'Smooth', ayuda: 'Few layers and long fades.' },
      relampago: { nombre: 'Lightning', ayuda: 'Fast cuts: many more songs fit.' }
    },
    crear: 'Create mashup',
    conStems:
      'Vocals are separated from instruments with the karaoke add-on: the first time takes a few minutes per song; after that, nothing.',
    sinStems:
      'Without the karaoke add-on the vocals cannot be separated: the mashup will chain and fade whole fragments. You can install it in Settings.',
    pocasAnalizadas:
      'At least two songs with music analysis done are needed. NB Sound analyzes them in the background; come back in a while.',
    ocupado: 'A mashup is already being prepared. Wait for it to finish or cancel it.',
    error: 'The mashup could not be started. Try again.',
    etapas: {
      eligiendo: 'Choosing compatible songs',
      analizando: 'Analyzing rhythm and sections',
      separando: 'Separating vocals and instruments',
      componiendo: 'Composing',
      mezclando: 'Mixing the audio'
    } as Record<string, string>,
    deUnidades: (hechas: number, total: number) => `${hechas} of ${total}`,
    porcentaje: (f: number) => `${Math.round(f * 100)}%`,
    cancelar: 'Cancel',
    estados: {
      generando: 'Being prepared',
      lista: 'Ready',
      fallida: 'Could not be created',
      cancelada: 'Canceled'
    } as Record<string, string>,
    errores: {
      INVALID_INPUT:
        'There are not enough songs that fit together. Try more songs, another playlist or a shorter style.',
      FILE_NOT_FOUND: 'Some songs are missing their files.',
      COMPONENT_UNAVAILABLE: 'The vocal separator is not available.',
      STORAGE_FAILURE: 'There is no room on the disk or it could not be written.',
      INTERNAL: 'It was interrupted before finishing.'
    } as Record<string, string>,
    resumen: (canciones: number, duracion: string) =>
      `${canciones === 1 ? '1 song' : `${canciones} songs`} · ${duracion}`,
    tempo: (bpm: number) => `${Math.round(bpm)} BPM`,
    tonalidad: (nota: string, menor: boolean) => `${nota} ${menor ? 'minor' : 'major'}`,
    escuchar: 'Listen',
    pausar: 'Pause',
    variante: 'Another variant',
    exportar: 'Export',
    exportado: 'Mashup exported.',
    noExportable: 'It cannot be exported inside your music folder.',
    eliminar: 'Delete',
    reintentar: 'Try again',
    lineaDeTiempo: 'Timeline',
    pistas: 'Songs in the mashup',
    carriles: {
      base: 'Beat',
      voz: 'Vocals',
      bateria: 'Drums'
    },
    materiales: {
      mezcla: 'whole song',
      instrumental: 'instrumental',
      sin_bateria: 'instrumental without drums',
      voz: 'vocals',
      bateria: 'drums'
    } as Record<string, string>,
    bloque: (titulo: string, material: string, inicio: string) =>
      `${titulo}, ${material}, from ${inicio}. Listen from here`,
    degradaciones: {
      no_caben: (pedidas: number, caben: number, minimo: string) =>
        `This length fits about ${caben} recognisable songs; ${pedidas} would need about ${minimo}.`,
      sin_stems: 'Vocals not separated: whole fragments chained.',
      menos_canciones: (pedidas: number, usadas: number) =>
        `You asked for ${pedidas} songs; ${usadas} fit well.`,
      frases_prolongadas: (n: number) =>
        `${n === 1 ? 'One phrase extends' : `${n} phrases extend`} the previous one for lack of good combinations.`,
      mas_corto: (seg: string) =>
        `It lasts ${seg}: there was no material for more without repeating.`
    }
  },
  pedir: {
    etiqueta: 'What do you want to hear?',
    ejemplo: 'For example: something calm for working at night, no reggaeton',
    duracion: 'Length',
    minutos: (n: number) => `${n} min`,
    crear: 'Create session',
    creando: 'Preparing the session…',
    error: 'The session could not be generated. Try again.',
    vacia: 'Type what you want to hear.',
    sinCandidatas: 'No song in your library fits that. Try other words or fewer exclusions.',
    ideas: 'Ideas',
    categoriasSugeridas: [
      {
        nombre: 'Vocals',
        chips: ['Female vocals', 'Male vocals', 'Instrumental', 'Acoustic']
      },
      {
        nombre: 'Genres',
        chips: [
          'Reggaeton',
          'Trap',
          'Pop',
          'Salsa',
          'Bachata',
          'Cumbia',
          'Rock',
          'Electronic',
          'Hip hop',
          'R&B',
          'Indie',
          'Jazz',
          'Classical'
        ]
      },
      {
        nombre: 'Moods',
        chips: ['Happy', 'Romantic', 'Melancholic', 'Nostalgic', 'Late night', 'Cinematic', 'Calm']
      },
      {
        nombre: 'Energy',
        chips: ['Energetic', 'Hard-hitting', 'Smooth', 'Relaxing', 'Progressive build']
      },
      {
        nombre: 'Moments',
        chips: ['Party', 'For working out', 'To focus', 'For cooking', 'Rainy night']
      }
    ],
    sugerencias: [
      'Progressive build for working out',
      'Something cinematic with female vocals',
      'Calm music to focus',
      'Electronic party with lots of energy',
      'Melancholic for a rainy night'
    ]
  },
  sesion: {
    entendi: 'I understood',
    evitare: 'Without',
    conceptos: {
      pop: 'pop',
      rock: 'rock',
      indie: 'indie',
      electronica: 'electronic',
      edm_agresivo: 'aggressive EDM',
      house: 'house',
      techno: 'techno',
      hip_hop: 'hip hop',
      reggaeton: 'reggaeton',
      reggae: 'reggae',
      salsa: 'salsa',
      bachata: 'bachata',
      merengue: 'merengue',
      cumbia: 'cumbia',
      trap: 'trap',
      rnb: 'R&B',
      jazz: 'jazz',
      clasica: 'classical',
      ambient: 'ambient',
      folk: 'folk',
      metal: 'metal',
      blues: 'blues',
      funk: 'funk',
      cinematografico: 'cinematic',
      orquestal: 'orchestral',
      narrativo: 'storytelling',
      sonador: 'dreamy',
      nocturno: 'late night',
      elegante: 'elegant',
      minimalista: 'minimalist',
      triste: 'sad',
      feliz: 'happy',
      esperanzador: 'hopeful',
      tranquilo: 'calm',
      agresivo: 'aggressive',
      energetico: 'energetic',
      etereo_calido: 'warm',
      voces_femeninas: 'female vocals',
      voces_masculinas: 'male vocals',
      instrumental: 'instrumental',
      que_resalten_voces: 'vocals up front',
      bajos_fuertes: 'strong bass',
      acustico: 'acoustic',
      electronico_modifier: 'electronic',
      oscuro: 'dark',
      brillante: 'bright',
      transiciones_suaves: 'smooth transitions',
      transiciones_agresivas: 'abrupt transitions',
      transicion_progresiva: 'progressive transition',
      entrenar: 'for working out',
      concentrarse: 'to focus',
      club_party: 'party',
      cocinar: 'for cooking',
      lluvioso: 'rainy',
      lo_fi: 'lo-fi',
      romantico: 'romantic',
      sin_edm_agresivo: 'no aggressive EDM',
      emocionante: 'exciting',
      sentimental: 'sentimental',
      melancolico_profundo: 'melancholic',
      nostalgico: 'nostalgic',
      dramatico: 'dramatic',
      epico: 'epic',
      que_pegue: 'hard-hitting',
      suave: 'smooth',
      relajante: 'relaxing',
      atmosferico: 'atmospheric',
      etereo: 'ethereal',
      espacial: 'spacey',
      futurista: 'futuristic',
      synthwave: 'synthwave',
      underground: 'underground',
      pesado: 'heavy',
      minimal_textural: 'minimal',
      bajos_profundos: 'deep bass',
      percusion_fuerte: 'strong percussion',
      vocal_dominante: 'vocals in the lead',
      voz_femenina_suave: 'soft female voice',
      voz_masculina_grave: 'deep male voice'
    } as Record<string, string>,
    curvas: {
      progressive: 'progressive build',
      wave: 'energy in waves',
      descending: 'falling energy',
      peak: 'energy peak'
    } as Record<string, string>,
    general: 'I did not recognize anything specific: this is a general selection of your music.',
    sonando: 'Playing',
    enPausa: 'Paused',
    construyendo: 'Preparing more songs while it plays…',
    resumen: (pistas: number, duracion: string, artistas: number) =>
      `${pistas === 1 ? '1 song' : `${pistas} songs`} · ${duracion} · ${artistas === 1 ? '1 artist' : `${artistas} artists`}`,
    transiciones: (buenas: number, total: number) => `${buenas} of ${total} very clean transitions`,
    sinAnalisis:
      'Without music analysis, the DJ chooses with the basic traits of each song. With the analysis add-on, in Settings, it gets finer.',
    escuchar: 'Listen',
    regenerar: 'Another like it',
    guardar: 'Save as playlist',
    guardada: 'Saved to Playlists.',
    descartar: 'Discard',
    meGusta: 'I like it',
    noMeGusta: 'I do not like it',
    valorada: 'I will keep it in mind.',
    pistas: 'Songs in the session',
    tocarDesde: (titulo: string) => `Listen from ${titulo}`,
    entrada: (tiempo: string) => `In ${tiempo}`,
    salida: (tiempo: string) => `Out ${tiempo}`,
    mezclando: (tecnica: string, progreso: number) => `${tecnica} · ${Math.round(progreso * 100)}%`,
    tecnicas: {
      hard_cut: 'Cut on the beat',
      energy_blend: 'Long fade',
      eq_kill_bass: 'Mixing with EQ',
      filter_sweep: 'Filter sweep',
      harmonic_mix: 'Blending layers'
    } satisfies Record<Tecnica, string>,
    transicion: {
      limpia: 'Clean transition',
      aceptable: 'Acceptable transition',
      forzada: 'Forced transition'
    }
  },
  historial: {
    titulo: 'History',
    buscar: 'Search the history',
    vacio: 'You have not asked for any session or mashup yet.',
    pocas: 'Your sessions and mashups stay here.',
    adornoTexto: 'Go back to any of them to listen again, see its songs, save it or export it.',
    nada: 'Nothing matches.',
    tipos: { sesion: 'Session', mashup: 'Mashup' },
    eliminarMashup: (titulo: string) => `Delete the mashup “${titulo}”`,
    estados: {
      construyendo: 'Being prepared',
      lista: 'Ready',
      finalizada: 'Played',
      descartada: 'Discarded',
      error: 'No songs'
    } as Record<string, string>,
    resumen: (pistas: number, duracion: string) => `${pistas} songs · ${duracion}`,
    ver: 'View',
    eliminar: 'Delete',
    eliminarSesion: (peticion: string) => `Delete the session “${peticion}”`
  }
};
