/**
 * Textos del DJ Privado. Van aparte del arranque, como `es-aparte.ts`:
 * solo los importa la vista del DJ, que se carga a demanda.
 */
import type { Tecnica } from '#lib/bindings/index.ts';

export const esDj = {
  titulo: 'DJ Privado',
  subtitulo: 'Dile qué te apetece y arma con tu música una sesión mezclada como la haría un DJ.',
  // El botón dice adónde lleva, no dónde se está: el título ya dice lo segundo.
  // Cerrar solo quita de la vista: lo cerrado sigue en el historial y, si se
  // estaba preparando, se sigue preparando.
  cerrar: 'Cerrar (sigue en el historial)',
  modos: {
    aMashups: 'DJ Mashups',
    aPrivado: 'DJ Privado',
    irAMashups: 'Cambiar a DJ Mashups',
    irAPrivado: 'Cambiar a DJ Privado'
  },
  mashups: {
    titulo: 'DJ Mashups',
    subtitulo:
      'Compone una pieza nueva con fragmentos de tus canciones: voces de unas sobre las bases de otras, al mismo tempo y en tonalidades que encajan.',
    origen: 'Canciones',
    toda: 'Toda tu biblioteca',
    playlist: (nombre: string) => `Playlist: ${nombre}`,
    cuantas: 'Cuántas',
    duracion: 'Duración',
    minutos: (n: number) => `${n} min`,
    estilo: 'Estilo',
    estilos: {
      equilibrado: {
        nombre: 'Equilibrado',
        ayuda: 'Frases de ocho compases y una subida en arco.'
      },
      fiesta: { nombre: 'Fiesta', ayuda: 'Más capas y energía que crece hasta el final.' },
      suave: { nombre: 'Suave', ayuda: 'Pocas capas y fundidos largos.' },
      relampago: { nombre: 'Relámpago', ayuda: 'Cortes rápidos: caben muchas más canciones.' }
    },
    crear: 'Crear mashup',
    conStems:
      'Las voces se separan de los instrumentos con el karaoke: la primera vez tarda unos minutos por canción; después, nada.',
    sinStems:
      'Sin el agregado de karaoke no se pueden separar las voces: el mashup se hará encadenando y fundiendo fragmentos enteros. Puedes instalarlo en Ajustes.',
    pocasAnalizadas:
      'Hacen falta al menos dos canciones con el análisis musical hecho. NB Sound las analiza en segundo plano; vuelve en un rato.',
    ocupado: 'Ya hay un mashup preparándose. Espera a que termine o cancélalo.',
    error: 'No se pudo empezar el mashup. Inténtalo de nuevo.',
    etapas: {
      eligiendo: 'Eligiendo canciones compatibles',
      analizando: 'Analizando el ritmo y las secciones',
      separando: 'Separando voces e instrumentos',
      componiendo: 'Componiendo',
      mezclando: 'Mezclando el audio'
    } as Record<string, string>,
    deUnidades: (hechas: number, total: number) => `${hechas} de ${total}`,
    porcentaje: (f: number) => `${Math.round(f * 100)} %`,
    cancelar: 'Cancelar',
    estados: {
      generando: 'Preparándose',
      lista: 'Lista',
      fallida: 'No se pudo crear',
      cancelada: 'Cancelada'
    } as Record<string, string>,
    errores: {
      INVALID_INPUT:
        'No hay suficientes canciones que encajen entre sí. Prueba con más canciones, otra playlist o un estilo más corto.',
      FILE_NOT_FOUND: 'Faltan archivos de algunas canciones.',
      COMPONENT_UNAVAILABLE: 'El separador de voces no está disponible.',
      STORAGE_FAILURE: 'No hay sitio en el disco o no se pudo escribir.',
      INTERNAL: 'Se interrumpió antes de terminar.'
    } as Record<string, string>,
    resumen: (canciones: number, duracion: string) =>
      `${canciones === 1 ? '1 canción' : `${canciones} canciones`} · ${duracion}`,
    tempo: (bpm: number) => `${Math.round(bpm)} BPM`,
    tonalidad: (nota: string, menor: boolean) => `${nota} ${menor ? 'menor' : 'mayor'}`,
    escuchar: 'Escuchar',
    pausar: 'Pausar',
    variante: 'Otra variante',
    exportar: 'Exportar',
    exportado: 'Mashup exportado.',
    noExportable: 'No se puede exportar dentro de la carpeta de tu música.',
    eliminar: 'Eliminar',
    reintentar: 'Volver a intentarlo',
    lineaDeTiempo: 'Línea de tiempo',
    pistas: 'Canciones del mashup',
    carriles: {
      base: 'Base',
      voz: 'Voz',
      bateria: 'Batería'
    },
    materiales: {
      mezcla: 'canción entera',
      instrumental: 'instrumental',
      sin_bateria: 'instrumental sin batería',
      voz: 'voz',
      bateria: 'batería'
    } as Record<string, string>,
    bloque: (titulo: string, material: string, inicio: string) =>
      `${titulo}, ${material}, desde ${inicio}. Escuchar desde aquí`,
    degradaciones: {
      no_caben: (pedidas: number, caben: number, minimo: string) =>
        `En esta duración caben unas ${caben} canciones reconocibles; para ${pedidas} harían falta unos ${minimo}.`,
      sin_stems: 'Sin separar voces: fragmentos enteros encadenados.',
      menos_canciones: (pedidas: number, usadas: number) =>
        `Pediste ${pedidas} canciones; encajaban bien ${usadas}.`,
      frases_prolongadas: (n: number) =>
        `${n === 1 ? 'Una frase alarga' : `${n} frases alargan`} la anterior por falta de combinaciones buenas.`,
      mas_corto: (seg: string) => `Dura ${seg}: no había material para más sin repetir.`
    }
  },
  pedir: {
    etiqueta: '¿Qué quieres escuchar?',
    ejemplo: 'Por ejemplo: algo tranquilo para trabajar de noche, sin reguetón',
    duracion: 'Duración',
    minutos: (n: number) => `${n} min`,
    crear: 'Crear sesión',
    creando: 'Preparando la sesión…',
    error: 'No se pudo generar la sesión. Inténtalo de nuevo.',
    vacia: 'Escribe lo que quieres escuchar.',
    sinCandidatas:
      'Ninguna canción de tu biblioteca encaja con eso. Prueba con otras palabras o con menos exclusiones.',
    ideas: 'Ideas',
    // Cada ficha es una palabra que el motor entiende: lo comprueba
    // `dj::ontologia::pruebas::las_fichas_de_la_vista_las_entiende_el_motor`.
    categoriasSugeridas: [
      {
        nombre: 'Voces',
        chips: ['Voces femeninas', 'Voces masculinas', 'Instrumental', 'Acústico']
      },
      {
        nombre: 'Géneros',
        chips: [
          'Reguetón',
          'Trap',
          'Pop',
          'Salsa',
          'Bachata',
          'Cumbia',
          'Rock',
          'Electrónica',
          'Hip hop',
          'R&B',
          'Indie',
          'Jazz',
          'Clásica'
        ]
      },
      {
        nombre: 'Ánimos',
        chips: [
          'Alegre',
          'Romántico',
          'Melancólico',
          'Nostálgico',
          'Nocturno',
          'Cinematográfico',
          'Tranquilo'
        ]
      },
      {
        nombre: 'Energía',
        chips: ['Energético', 'Que pegue', 'Suave', 'Relajante', 'Subida progresiva']
      },
      {
        nombre: 'Momentos',
        chips: ['Fiesta', 'Para entrenar', 'Para concentrarme', 'Para cocinar', 'Noche de lluvia']
      }
    ],
    sugerencias: [
      'Subida progresiva para entrenar',
      'Algo cinematográfico con voces femeninas',
      'Tranquilo para concentrarme',
      'Fiesta electrónica con mucha energía',
      'Melancólico para una noche de lluvia'
    ]
  },
  sesion: {
    entendi: 'Entendí',
    evitare: 'Sin',
    // Los conceptos de la ontología (`dj/ontologia_datos.rs`) por su nombre, para
    // decir cómo se entendió el pedido. Lo comprueba
    // `dj::intencion::tests::cada_concepto_tiene_su_nombre_en_la_vista`.
    conceptos: {
      pop: 'pop',
      rock: 'rock',
      indie: 'indie',
      electronica: 'electrónica',
      edm_agresivo: 'EDM agresivo',
      house: 'house',
      techno: 'techno',
      hip_hop: 'hip hop',
      reggaeton: 'reguetón',
      reggae: 'reggae',
      salsa: 'salsa',
      bachata: 'bachata',
      merengue: 'merengue',
      cumbia: 'cumbia',
      trap: 'trap',
      rnb: 'R&B',
      jazz: 'jazz',
      clasica: 'clásica',
      ambient: 'ambient',
      folk: 'folk',
      metal: 'metal',
      blues: 'blues',
      funk: 'funk',
      cinematografico: 'cinematográfica',
      orquestal: 'orquestal',
      narrativo: 'que cuente una historia',
      sonador: 'soñadora',
      nocturno: 'nocturna',
      elegante: 'elegante',
      minimalista: 'minimalista',
      triste: 'triste',
      feliz: 'alegre',
      esperanzador: 'esperanzadora',
      tranquilo: 'tranquila',
      agresivo: 'agresiva',
      energetico: 'con energía',
      etereo_calido: 'cálida',
      voces_femeninas: 'voces femeninas',
      voces_masculinas: 'voces masculinas',
      instrumental: 'instrumental',
      que_resalten_voces: 'voces al frente',
      bajos_fuertes: 'bajos fuertes',
      acustico: 'acústica',
      electronico_modifier: 'electrónica',
      oscuro: 'oscura',
      brillante: 'brillante',
      transiciones_suaves: 'transiciones suaves',
      transiciones_agresivas: 'transiciones bruscas',
      transicion_progresiva: 'transición progresiva',
      entrenar: 'para entrenar',
      concentrarse: 'para concentrarse',
      club_party: 'fiesta',
      cocinar: 'para cocinar',
      lluvioso: 'de lluvia',
      lo_fi: 'lo-fi',
      romantico: 'romántica',
      sin_edm_agresivo: 'sin EDM agresivo',
      emocionante: 'emocionante',
      sentimental: 'sentimental',
      melancolico_profundo: 'melancólica',
      nostalgico: 'nostálgica',
      dramatico: 'dramática',
      epico: 'épica',
      que_pegue: 'que pegue',
      suave: 'suave',
      relajante: 'relajante',
      atmosferico: 'atmosférica',
      etereo: 'etérea',
      espacial: 'espacial',
      futurista: 'futurista',
      synthwave: 'synthwave',
      underground: 'underground',
      pesado: 'pesada',
      minimal_textural: 'minimal',
      bajos_profundos: 'bajos profundos',
      percusion_fuerte: 'percusión fuerte',
      vocal_dominante: 'la voz como protagonista',
      voz_femenina_suave: 'voz femenina suave',
      voz_masculina_grave: 'voz masculina grave'
    } as Record<string, string>,
    curvas: {
      progressive: 'subida progresiva',
      wave: 'energía en olas',
      descending: 'energía que baja',
      peak: 'pico de energía'
    } as Record<string, string>,
    general: 'No reconocí nada concreto: es una selección general de tu música.',
    sonando: 'Sonando',
    enPausa: 'En pausa',
    construyendo: 'Preparando más canciones mientras suena…',
    resumen: (pistas: number, duracion: string, artistas: number) =>
      `${pistas === 1 ? '1 canción' : `${pistas} canciones`} · ${duracion} · ${artistas === 1 ? '1 artista' : `${artistas} artistas`}`,
    transiciones: (buenas: number, total: number) =>
      `${buenas} de ${total} transiciones muy limpias`,
    sinAnalisis:
      'Sin el análisis musical, el DJ elige con los rasgos básicos de cada canción. Con el agregado de análisis, en Ajustes, afina más.',
    escuchar: 'Escuchar',
    regenerar: 'Otra parecida',
    guardar: 'Guardar como playlist',
    guardada: 'Guardada en Playlists.',
    descartar: 'Descartar',
    meGusta: 'Me gusta',
    noMeGusta: 'No me gusta',
    valorada: 'Lo tendré en cuenta.',
    pistas: 'Canciones de la sesión',
    tocarDesde: (titulo: string) => `Escuchar desde ${titulo}`,
    entrada: (tiempo: string) => `Entrada ${tiempo}`,
    salida: (tiempo: string) => `Salida ${tiempo}`,
    mezclando: (tecnica: string, progreso: number) =>
      `${tecnica} · ${Math.round(progreso * 100)} %`,
    tecnicas: {
      hard_cut: 'Corte en el beat',
      energy_blend: 'Fundido largo',
      eq_kill_bass: 'Mezclando con ecualización',
      filter_sweep: 'Barrido de filtros',
      harmonic_mix: 'Fundiendo capas'
    } satisfies Record<Tecnica, string>,
    transicion: {
      limpia: 'Transición limpia',
      aceptable: 'Transición aceptable',
      forzada: 'Transición forzada'
    }
  },
  historial: {
    titulo: 'Historial',
    buscar: 'Buscar en el historial',
    vacio: 'Aún no has pedido ninguna sesión ni ningún mashup.',
    pocas: 'Tus sesiones y mashups se quedan aquí.',
    adornoTexto:
      'Vuelve a cualquiera para escucharlo otra vez, ver sus canciones, guardarlo o exportarlo.',
    nada: 'Nada coincide.',
    tipos: { sesion: 'Sesión', mashup: 'Mashup' },
    eliminarMashup: (titulo: string) => `Eliminar el mashup «${titulo}»`,
    estados: {
      construyendo: 'Preparándose',
      lista: 'Lista',
      finalizada: 'Escuchada',
      descartada: 'Descartada',
      error: 'Sin canciones'
    } as Record<string, string>,
    resumen: (pistas: number, duracion: string) => `${pistas} canciones · ${duracion}`,
    ver: 'Ver',
    eliminar: 'Eliminar',
    eliminarSesion: (peticion: string) => `Eliminar la sesión «${peticion}»`
  }
} as const;

export type TextosDj = typeof esDj;
