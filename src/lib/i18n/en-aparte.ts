/**
 * Textos en inglés de las pantallas que se cargan aparte del arranque.
 */
import type { TextosAparte } from './es-aparte';
import type { Forma } from './forma';

const n = (v: number) => v.toLocaleString('en');
const canciones = (v: number) => (v === 1 ? '1 song' : `${n(v)} songs`);

export const enAparte: Forma<TextosAparte> = {
  comun: {
    error: 'It could not be completed. Try again.'
  },
  apariencia: {
    idioma: 'Language',
    idiomaAyuda: 'Changes the whole interface at once, without restarting.',
    preseleccionesEq: [
      'Flat',
      'Classical',
      'Club',
      'Dance',
      'Bass',
      'Bass and treble',
      'Treble',
      'Headphones',
      'Large hall',
      'Live',
      'Party',
      'Pop',
      'Reggae',
      'Rock',
      'Ska',
      'Soft',
      'Soft rock',
      'Techno'
    ],
    nombresTemas: {
      negro_puro: 'Pure Black (OLED)',
      arcilla_nocturna: 'Night Clay',
      arcilla_calida: 'Warm Clay',
      oro_liquido: 'Liquid Gold',
      oscuro_profundo: 'Deep Abyss',
      electrico_voltaje: 'Blue Voltage',
      lima_suave: 'Laser Lime',
      carmesi_nocturno: 'Night Crimson',
      aurora_boreal: 'Northern Lights',
      selva_neon: 'Neon Jungle',
      crepusculo_violeta: 'Violet Dusk',
      cobalto_night: 'Night Cobalt',
      menta_fresh: 'Fresh Mint',
      carbono: 'Cold Carbon',
      vampiro_royal: 'Royal Vampire',
      ambar_calido: 'Warm Amber',
      nieve: 'Snow',
      ocean_profundo: 'Abyssal Ocean',
      galaxia: 'Purple Galaxy',
      desierto_dorado: 'Golden Desert',
      bosque_mistico: 'Mystic Forest',
      fuego_solar: 'Solar Fire',
      sakura_nocturno: 'Night Sakura',
      glacial: 'Blue Glacier',
      terminal_verde: 'Green Terminal',
      sangre_dragon: 'Dragon Blood',
      prisma: 'Prism',
      lava: 'Black Lava',
      sepia: 'Imperial Sepia',
      medianoche: 'Midnight Blue',
      esmeralda: 'Emerald',
      titanio: 'Titanium',
      coral_sunrise: 'Coral Sunrise',
      indigo_profundo: 'Deep Indigo',
      aurora_rosa: 'Pink Aurora',
      cafe_nocturno: 'Night Coffee',
      obsidiana_neon: 'Neon Obsidian',
      hielo_oled: 'OLED Ice',
      blanco_editorial: 'Editorial White',
      oro_negro: 'Black Gold',
      jade_nocturno: 'Night Jade',
      plasma_morado: 'Purple Plasma',
      amanecer_sintetico: 'Synthetic Dawn',
      tinta_marina: 'Marine Ink',
      frambuesa_dark: 'Dark Raspberry',
      cielo_coral: 'Coral Sky',
      violeta_laser: 'Laser Violet',
      acero_azul: 'Blue Steel',
      rosa_polar: 'Polar Pink',
      circuito_verde: 'Green Circuit',
      ultra_violeta: 'Ultraviolet',
      marfil_grafito: 'Ivory Graphite',
      noche_arcade: 'Arcade Night',
      neon_citrico: 'Citrus Neon',
      cosmic_latte: 'Cosmic Latte',
      turquesa_electrica: 'Electric Turquoise',
      magma_violeta: 'Violet Magma',
      bruma_artica: 'Arctic Mist',
      cobre_oxidado: 'Oxidized Copper',
      sombra_lima: 'Lime Shadow',
      zafiro_medianoche: 'Midnight Sapphire',
      rosa_quimera: 'Chimera Pink',
      grafito_radioactivo: 'Radioactive Graphite'
    } as Record<string, string>
  },
  bienvenida: {
    titulo: 'Your music, at its best',
    subtitulo:
      'NB Sound organizes your library, fills in covers and lyrics, and makes it sound the way it deserves. All on this computer.',
    nombre: 'What should we call you?',
    nombreMarcador: 'Your name',
    carpetaTitulo: 'I have a folder with my music',
    carpetaTexto: 'NB Sound imports all of it and keeps an eye on what you copy into it later.',
    carpetaBoton: 'Choose the folder',
    sueltaTitulo: 'I want to add individual songs',
    sueltaTexto: 'Choose files or drag them onto the window whenever you like.',
    sueltaBoton: 'Add songs',
    copiaTitulo: 'I am coming from another computer',
    copiaTexto: 'Restore an NB Sound backup: library, playlists and history.',
    copiaBoton: 'Restore a backup',
    buscando: 'Looking for songs in your folder. You can keep using NB Sound meanwhile.',
    restaurada: 'Backup restored. NB Sound restarts to apply it.'
  },
  dondeVive: {
    titulo: 'Where your music lives',
    ayuda:
      'If you move the folder, reformat or come from another computer, NB Sound looks for it on its own at startup. If it cannot find it, it asks you where it is.',
    sinCarpeta: 'You do not have a music folder yet: you choose it when importing.',
    todas: (v: number) => (v === 1 ? 'Your song is here.' : `Your ${n(v)} songs are here.`),
    faltan: (faltan: number, total: number) =>
      `${n(faltan)} of your ${n(total)} songs ${faltan === 1 ? 'is' : 'are'} missing.`,
    perdida: 'Your music is not in this folder. Choose where it is now.',
    vacia: 'Your library has no songs yet.',
    comprobar: 'Check',
    elegir: 'Choose',
    comprobado: 'Checked: everything is in place.',
    preguntaTitulo: 'Where is your music?',
    pregunta: (total: number) =>
      `Your ${n(total)} songs were in this folder, but they are not there anymore. If you moved it, reformatted or came from another computer, tell me where it is now and everything will stay as you left it.`,
    buscar: 'Find my music',
    ahoraNo: 'Not now',
    recuperada: 'Your music moved and NB Sound found it on its own.',
    ninguna: 'None of your songs are there. Try another folder.'
  },
  carpetaMusica: {
    cambiar: 'Change',
    titulo: 'Change your music folder',
    misma: 'That is already your music folder.',
    yaEsta: (v: number) =>
      v === 1
        ? 'Your song is already there: NB Sound only has to point to the new folder.'
        : `Your ${n(v)} songs are already there: NB Sound only has to point to the new folder.`,
    faltan: (encontradas: number, total: number) =>
      encontradas === 0
        ? `None of your ${n(total)} songs are there. NB Sound can move them: each one to the same subfolder, and the originals are removed when it finishes.`
        : `${n(encontradas)} of your ${n(total)} songs are there. NB Sound can move the rest: each one to the same subfolder.`,
    mover: 'Move my music',
    usar: 'Use this folder',
    otra: 'Choose another',
    parcial: (encontradas: number, total: number) =>
      `${n(encontradas)} of your ${n(total)} songs are there. The rest cannot be played until they are in that folder.`,
    usarSinMover: 'Use without moving',
    usarSinMoverAviso:
      'If you do not move the music, songs that are not in the new folder cannot be played until you move them yourself.',
    moviendo: (hechas: number, total: number) => `Moving your music… ${n(hechas)} of ${n(total)}`,
    hecho: 'Your music folder is now the new one.',
    movidas: (v: number) =>
      v === 1 ? 'One song moved to the new folder.' : `${n(v)} songs moved to the new folder.`,
    fallo:
      'Your music could not be moved (space, permissions?). Everything is back in place and the folder has not changed.',
    dentro: 'The new folder cannot be inside the current one, or the other way round.'
  },
  descubrir: {
    titulo: (que: string) => `For what you asked: ${que}`,
    sinAnalisis:
      'No songs have been analyzed yet. They are analyzed on their own in the background after import.',
    nada: 'No song fits what you asked closely enough.',
    intenciones: {
      triste: 'sad',
      alegre: 'happy',
      fiesta: 'for a party',
      entrenar: 'for working out',
      concentrarse: 'to focus',
      tranquila: 'calm',
      nocturna: 'late night',
      intensa: 'intense',
      suave: 'soft',
      rapida: 'fast',
      lenta: 'slow'
    },
    secciones: {
      masFiesteras: 'Most party-ready',
      masBailables: 'Most danceable',
      masRapidas: 'Fastest',
      energiaAlta: 'High energy',
      masMelancolicas: 'Most melancholic',
      tristesSuaves: 'Sad but soft',
      tristesConEnergia: 'Sad with more energy',
      masIntensas: 'Most intense',
      ritmoAlto: 'Fast rhythm',
      energiaParaMoverse: 'Energy to move',
      masEnfocadas: 'Most focused',
      tranquilasParaTrabajar: 'Calm for working',
      bajaDistraccion: 'Low distraction',
      masNocturnas: 'Most nocturnal',
      oscurasYCalmadas: 'Dark and calm',
      suavesDeNoche: 'Soft at night',
      masTranquilas: 'Calmest',
      suavesParaBajarRitmo: 'Soft to slow down',
      paraLevantarAnimo: 'To lift your mood',
      alegresYLigeras: 'Happy and light',
      masSuaves: 'Softest',
      bajaEnergia: 'Low energy',
      masLentas: 'Slowest',
      bajaVelocidad: 'Low tempo',
      seleccionSugerida: 'Suggested selection'
    } as Record<string, string>
  },
  agregados: {
    ayuda:
      'Large, optional features. They download from the NB Sound releases on GitHub with one click, and removing them frees their space at once.',
    nombres: {
      karaoke: 'Karaoke',
      analisis: 'Music analysis',
      textoDj: 'DJ requests in your own words',
      aceleracion: 'GPU acceleration'
    },
    descripciones: {
      karaoke:
        'Separates the vocals from any song so you can sing it, and prepares instrumentals for the phone and the DJ.',
      analisis:
        'Recognizes the mood, danceability and style of each song for smart playlists and the DJ.',
      textoDj: 'Understands what you ask the DJ in your own words.',
      aceleracion:
        'Makes karaoke and analysis several times faster with an NVIDIA graphics card. Applies the next time NB Sound opens.'
    },
    tamano: (megas: number) => `${megas} MB`,
    instalar: (megas: number) => `Install (${megas} MB)`,
    descargando: (porcentaje: number) => `Downloading ${porcentaje}%`,
    instalado: 'Installed',
    actualizar: 'Update',
    quitar: 'Remove',
    noDisponible: 'Not available on this computer',
    ofertaNecesita: (megas: number) =>
      `Needs a ${megas} MB add-on. It downloads once and you can keep listening meanwhile.`,
    ofertaSigue: 'you can keep listening',
    ofertaLista: 'Installed. Press the microphone to prepare this song.',
    cerrar: 'Close',
    recomendado: (gpu: string) => `Recommended: your computer has ${gpu}.`,
    gpus: {
      cuda: 'an NVIDIA card'
    },
    sinConexion: 'Turn on online services in Privacy to download it.',
    errores: {
      NETWORK_FAILURE:
        'The download was interrupted. Press Install again to resume where it stopped.',
      INTEGRITY_MISMATCH:
        'The downloaded file is not the expected one and was discarded. Try again.',
      STORAGE_FAILURE: 'There is no space or the disk could not be written.',
      NETWORK_DISABLED: 'Turn on online services in Privacy to download it.'
    } as Record<string, string>,
    error: 'It could not be installed. Try again.'
  },
  actualizacion: {
    buscar: 'Check for updates',
    buscando: 'Checking',
    alDia: 'You have the latest version.',
    disponible: (v: string) => `NB Sound ${v} is available.`,
    instalar: 'Install and restart',
    instalando: 'Downloading and installing. NB Sound will restart on its own.',
    sinConexion: 'Turn on online services in Privacy to check for updates.',
    noDisponible:
      'This installation is updated with the system package manager or by downloading the new version.',
    firma: 'The download does not carry the NB Sound signature, so it was not installed.',
    administrada: 'This installation is updated from NB Launcher.',
    error: 'Could not connect. Try again later.'
  },
  copia: {
    titulo: 'Backup',
    ayuda:
      'Saves your library, playlists, history, covers and your picture in a single file. It does not include the music or your keys.',
    crear: 'Create a backup now',
    creando: 'Creating the backup',
    creada: (megas: string, portadas: number) =>
      `Backup saved: ${megas} MB, with ${portadas === 1 ? '1 cover' : `${portadas} covers`}.`,
    restaurar: 'Restore a backup',
    confirmarTitulo: 'Restore a backup?',
    confirmarTexto:
      'Your current library will be replaced by the one in the backup when NB Sound restarts. The current one is set aside in case you need to go back. Your music is not touched.',
    elegirCopia: 'Choose the backup',
    cancelar: 'Cancel',
    pendiente: 'The backup is ready. It will be applied when NB Sound restarts.',
    reiniciar: 'Restart now',
    errorFormato: 'This file is not a backup from this version of NB Sound.',
    errorNueva:
      'The backup is from a newer version of NB Sound. Update the application to restore it.',
    errorDanada: 'The backup is damaged or incomplete.',
    error: 'It could not be completed. Try again.',
    automaticas: 'Automatic backups',
    conflicto:
      'The frequency was changed from another control. The saved value is shown; choose again if you want to change it.',
    frecuencias: { 0: 'Never', 1: 'Daily', 7: 'Weekly', 30: 'Monthly' } as Record<number, string>,
    carpeta: 'Backup folder',
    sinCarpeta: 'Choose a folder so they can be made.',
    elegirCarpeta: 'Choose folder',
    ultima: (fecha: string) => `Last backup: ${fecha}`,
    ninguna: 'No backup has been made yet.'
  },
  edicion: {
    tituloPista: 'Edit song details',
    tituloAlbum: 'Edit album details',
    ayuda:
      'Saved in your library and, when possible, in the file. Playlists and the phone keep recognizing it.',
    titulo: 'Title',
    artista: 'Artist',
    album: 'Album',
    albumAyuda: 'Empty if it does not belong to an album.',
    numero: 'Track number',
    anio: 'Year',
    genero: 'Genre',
    guardar: 'Save',
    cancelar: 'Cancel',
    invalido:
      'Check the details: title and artist cannot be empty, the track number goes from 1 to 999 and the year from 1000 to 9999.',
    sinArchivo: 'This song’s file cannot be found.',
    fallo: 'It could not be saved. Try again.'
  },
  privacidad: {
    privacidadAyuda:
      'NB Sound works offline. Each service is only used if you turn it on, and it only receives what is stated.',
    conexion: 'Use online services',
    conexionAyuda: 'When off, nothing leaves this computer, whatever each service is set to.',
    servicios: {
      acoustId: {
        nombre: 'AcoustID',
        envia: 'The acoustic fingerprint and length of each song, to recognize it. Never the audio.'
      },
      musicBrainz: {
        nombre: 'MusicBrainz',
        envia:
          'Title, artist and album, or the identifier AcoustID returned, to complete the details.'
      },
      coverArtArchive: {
        nombre: 'Cover Art Archive',
        envia: 'The album’s MusicBrainz identifier, to download its cover.'
      },
      itunes: {
        nombre: 'iTunes',
        envia: 'Title and artist, to confirm the song and look for covers.'
      },
      lrclib: {
        nombre: 'LRCLIB',
        envia: 'Title, artist, album and length, to look for synced lyrics.'
      },
      lyricsOvh: {
        nombre: 'lyrics.ovh',
        envia: 'Title and artist, to look for lyrics if LRCLIB does not have them.'
      },
      deezer: {
        nombre: 'Deezer',
        envia: 'The artist or album name, to look for images.'
      },
      wikipedia: {
        nombre: 'Wikipedia',
        envia: 'The artist name, to look for their photo.'
      },
      theAudioDb: {
        nombre: 'TheAudioDB',
        envia: 'The artist and album names, to look for covers and artist images.'
      },
      actualizaciones: {
        nombre: 'NB Sound updates',
        envia:
          'The installed version and the system, to GitHub, only when you check for a new version from About.'
      }
    },
    iaAyuda:
      'Optional. When two versions of a song tie, a language model picks between them using your own account. It only receives the details of the song and the candidates, never the audio.',
    iaActiva: 'Break ties with AI',
    iaSinConexion: 'Turn on online services first.',
    proveedor: 'Provider',
    proveedores: { anthropic: 'Anthropic (Claude)', openai: 'OpenAI' },
    modelo: 'Model',
    modeloAyuda: 'Leave it empty to use the default.',
    clave: 'API key',
    claveGuardada: 'Saved in the system keyring.',
    claveNueva: 'Paste your key here',
    guardarClave: 'Save key',
    olvidarClave: 'Forget key',
    sinLlavero:
      'This system has no password keyring, so no key can be saved. NB Sound never stores it in a file.',
    noGuardado: 'It could not be saved. Try again.',
    claveServicio: {
      acoustId:
        'Without it, a song cannot be recognized by its sound. It is free: request one at acoustid.org with an account.'
    }
  },
  inteligentes: {
    nueva: 'New smart playlist',
    editar: 'Edit rule',
    crearParaMi: 'Create for me',
    creadasParaTi: (v: number) =>
      v === 0
        ? 'Your playlists were already up to date with your music'
        : v === 1
          ? '1 playlist created or updated with your music'
          : `${n(v)} playlists created or updated with your music`,
    creandoParaMi: 'Preparing playlists',
    ayuda:
      'Updates itself every time you import music, mark favorites or play songs that meet these conditions.',
    animo: 'Mood',
    cualquierAnimo: 'Any',
    animos: {
      energetica: 'Energetic',
      tranquila: 'Calm',
      bailable: 'Danceable',
      triste: 'Sad',
      nocturna: 'Late night',
      entrenamiento: 'For working out',
      concentracion: 'For focusing',
      fiesta: 'For a party',
      feliz: 'Happy',
      relajada: 'Relaxed',
      intensa: 'Intense'
    } as Record<string, string>,
    bpmMin: 'Minimum BPM',
    bpmMax: 'Maximum BPM',
    energiaMin: 'Minimum energy (0 to 100%)',
    energiaMax: 'Maximum energy (0 to 100%)',
    instrumental: 'Karaoke instrumental',
    instrumentalOpciones: {
      cualquiera: 'Any',
      con: 'Only with instrumental ready',
      sin: 'Only without instrumental'
    },
    soloFavoritas: 'Only favorite songs',
    reproduccionesMin: 'Minimum plays',
    reproduccionesMax: 'Maximum plays',
    frecuencia: 'Last played',
    frecuenciaOpciones: {
      cualquiera: 'Any time',
      reciente: 'Played in the last days',
      olvidada: 'Not played for days'
    },
    dias: 'Days',
    anioMin: 'Year from',
    anioMax: 'Year to',
    genero: 'Genre contains',
    orden: 'Sort by',
    ordenes: {
      recientes: 'Most recently added',
      masEscuchadas: 'Most played',
      menosEscuchadas: 'Least played',
      ultimaEscuchaAsc: 'Longest without playing',
      bpmAsc: 'BPM low to high',
      bpmDesc: 'BPM high to low',
      energiaDesc: 'Highest energy',
      tituloAsc: 'Title (A to Z)'
    },
    limite: 'Maximum songs',
    invalida: 'Check the ranges: the minimum cannot exceed the maximum.'
  },

  resumenEscucha: {
    titulo: 'Local listening summary',
    sub: 'Your activity by month, by year or across your whole history, without leaving this computer.',
    todo: 'All history',
    exportar: 'Export SVG card',
    exportando: 'Exporting…',
    exportada: 'Card saved.',
    radarTitulo: 'Mood radar',
    radarSub: 'Average of the songs played in the period.',
    ejes: {
      energia: 'Energy',
      bailable: 'Danceable',
      alegre: 'Happy',
      calma: 'Calm',
      nocturna: 'Nocturnal',
      melancolica: 'Melancholic'
    },
    diasSemana: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
    mesesTitulo: 'Month by month',
    mesesSub: 'How much music you listened to each month. Tap one to see it day by day.',
    celdaMes: (mes: string, escuchas: number) =>
      `${mes} · ${escuchas} ${escuchas === 1 ? 'play' : 'plays'}`,
    calendarioTitulo: 'Your month, day by day',
    calendarioSub: 'How much music you listened to each day of the month.',
    celdaDia: (dia: number, escuchas: number) =>
      `Day ${dia} · ${escuchas} ${escuchas === 1 ? 'play' : 'plays'}`,
    reproduccionesPeriodo: (v: number) => `${n(v)} ${v === 1 ? 'play' : 'plays'}`,
    meses: {
      '01': 'Jan',
      '02': 'Feb',
      '03': 'Mar',
      '04': 'Apr',
      '05': 'May',
      '06': 'Jun',
      '07': 'Jul',
      '08': 'Aug',
      '09': 'Sep',
      '10': 'Oct',
      '11': 'Nov',
      '12': 'Dec'
    } as Record<string, string>
  },

  copiaDispositivo: {
    titulo: 'Copy to a card or USB drive',
    ayuda:
      'Exports your music with names valid for FAT32/exFAT, embedded cover and a .m3u8 playlist file.',
    origen: 'What to copy',
    origenes: {
      biblioteca: 'The whole library',
      favoritas: 'My favorite songs',
      playlist: 'A playlist'
    },
    playlist: 'Playlist to copy',
    sinPlaylists: 'No playlists created yet.',
    estructura: 'Folder structure',
    estructuras: {
      artistaAlbum: 'Artist / Album / 01 - Song',
      artista: 'Artist / 01 - Song',
      plana: 'Everything in one folder (Artist - Song)'
    },
    incrustarPortada: 'Embed cover and tags in the copies',
    convertirWav: 'Convert to WAV PCM (16-bit, 44.1 kHz) for maximum compatibility',
    botonCopiar: 'Choose folder and copy',
    copiando: 'Copying to the device…',
    resumen: (copiadas: number, omitidas: number, mb: string, m3u: string | null) =>
      `${n(copiadas)} ${copiadas === 1 ? 'track copied' : 'tracks copied'}${omitidas > 0 ? `, ${n(omitidas)} already there` : ''} (${mb} MB)${m3u ? ` · ${m3u}` : ''}.`,
    error: 'The copy to the device could not be completed.'
  },

  modoFiesta: {
    titulo: 'Party Mode',
    ayuda:
      'Your guests scan a code and suggest songs from their browser, without installing anything.',
    interruptor: 'Turn on Party Mode',
    interruptorAyuda:
      'It only works while it is on. Turning it off closes the port and deletes the sessions.',
    qrEtiqueta: 'QR code to join Party Mode',
    urlEtiqueta: 'Link for guests on the same network:',
    regenerarQr: 'Rotate QR',
    automatico: 'Queue suggestions automatically',
    automaticoAyuda:
      'When off, you approve or discard each suggestion before it goes to the queue.',
    sugerenciasTitulo: 'Guest suggestions',
    sinSugerencias: 'No suggestions have been sent yet.',
    porInvitado: (alias: string) => `Suggested by ${alias}`,
    enCola: 'In the queue',
    aceptar: 'Accept',
    descartar: 'Discard',
    bloquear: 'Block guest',
    bloqueados: (v: number) =>
      `${v} ${v === 1 ? 'guest blocked' : 'guests blocked'} in this session`
  },

  duplicados: {
    titulo: 'Duplicate songs',
    descripcion:
      'Copies NB Sound set aside because you already had the song. They are not shown and do not go to the phone, and their file stays where it is.',
    sinDuplicados: 'There are no copies set aside.',
    encontrados: (v: number, megas: number) =>
      v === 1 ? `1 copy set aside (${n(megas)} MB)` : `${n(v)} copies set aside (${n(megas)} MB)`,
    limpiar: 'Clean up duplicates',
    buscar: 'Find duplicates',
    buscando: 'Looking for copies…',
    apartadas: (v: number) =>
      v === 0
        ? 'There are no new copies in your library.'
        : v === 1
          ? '1 copy set aside: it is no longer shown in your library.'
          : `${n(v)} copies set aside: they are no longer shown in your library.`,
    confirmarTitulo: 'Clean up duplicate songs?',
    confirmarPregunta: (v: number) =>
      v === 1
        ? '1 copy will be removed from the library. The original song is not touched.'
        : `${n(v)} copies will be removed from the library. The original song is not touched.`,
    borrarArchivos: 'Also move their files to the trash',
    purgarBtn: 'Clean up',
    cancelar: 'Cancel',
    purgadas: (v: number) =>
      v === 1 ? '1 copy removed from the library' : `${n(v)} copies removed from the library`,
    error: 'The copies could not be cleaned up.'
  },

  buscar: {
    bienvenida: {
      titulo: 'Ask your library',
      descripcion:
        'Type a title, an artist, a line from the lyrics or what you feel like hearing. Try one of these:'
    },
    ideas: [
      'Happy songs to start the day',
      'Something calm for cooking',
      'Music to focus',
      'Energetic tracks for the gym',
      'Something sad for a rainy night',
      'Party music',
      'Soft songs for reading',
      'For driving at night',
      'Something chill to relax',
      'Intense and powerful tracks',
      'Danceable songs for cleaning the house',
      'Slow and romantic songs',
      'Dark late-night music',
      'Fast songs for running',
      'Nostalgia for the afternoon',
      'To study without distractions'
    ]
  },
  karaokeConsola: {
    titulo: 'Karaoke',
    subtitulo: 'Sing any song in your library. The vocals are separated on this computer, offline.',
    abrir: 'Open the karaoke console',
    motorNoInstalado: 'The karaoke add-on is missing',
    motorExplicacion: 'It removes the vocals from songs. It downloads once and works offline.',
    instalar: 'Install the add-on',
    ahora: 'Now playing',
    nadaSonando: 'Play a song to sing it.',
    lista: 'Ready to sing',
    noLista: 'Not prepared',
    preparandoEsta: (pct: number) => `Preparing · ${pct}%`,
    cantar: 'Sing now',
    dejarDeCantar: 'Back to the original',
    prepararPistaActual: 'Prepare this song',
    enPreparacion: 'Being prepared',
    enEspera: 'Waiting',
    fallidas: (v: number) =>
      v === 1
        ? '1 song could not be prepared. "Prepare my favorites" tries again if it is a favorite.'
        : `${v} songs could not be prepared. "Prepare my favorites" tries again with the favorites.`,
    yMas: (v: number) => `And ${v} more after that.`,
    siguePorSuCuenta:
      'They are prepared one after another while you listen. If you close NB Sound, it picks up where it left off when you open it again.',
    sinPendientes: 'Nothing being prepared. You can keep listening while they are prepared.',
    repertorio: 'Your repertoire',
    listas: (v: number) => `${canciones(v)} ready to sing`,
    verListas: 'See them in the library',
    prepararFavoritas: 'Prepare my favorites',
    prepararPlaylist: 'Prepare a playlist',
    elegirPlaylist: 'Playlist to prepare',
    playlistEnCola: (v: number) =>
      v === 0
        ? 'That playlist is already ready or on its way.'
        : `${canciones(v)} being prepared. You can keep listening.`
  },
  almacenamiento: {
    titulo: 'Storage',
    subtitulo:
      'What NB Sound keeps on this computer. Whatever can be rebuilt has its own limit: filling one category never deletes another.',
    categorias: {
      portadas: 'Covers and artist photos',
      miniaturas: 'Cover thumbnails',
      separaciones: 'Vocal and instrument separations',
      karaoke: 'Karaoke instrumentals',
      mashups: 'Created mashups',
      base_de_datos: 'Library, history and settings'
    },
    ayudas: {
      portadas:
        'The originals. They are not deleted: some came from services that may no longer provide them.',
      miniaturas: 'Rebuilt on their own when each cover is shown.',
      separaciones:
        'The vocals, drums, bass and the rest of each song, for karaoke and mashups. Once a song is separated, the next mashup or karaoke with it takes seconds.',
      karaoke: 'What plays when you sing and what the phone downloads.',
      mashups: 'The ones you created. They are only deleted if you delete them.',
      base_de_datos:
        'Your library, playlists, history and settings, with the backups from each update.'
    },
    usado: 'Uses',
    todaLaBiblioteca: (s: string) => `With your whole library: about ${s}`,
    limite: 'Limit',
    liberar: 'Free up',
    liberado: (s: string) => `Freed ${s}.`,
    confirmarLiberar: (cat: string, s: string) => `Empty ${cat}? This frees ${s}.`,
    rehacer: (t: string) => `Rebuilding it would take about ${t} of computer work.`,
    rehacerSolo: 'Rebuilt on their own when used.',
    total: (s: string) => `NB Sound uses ${s} in total`,
    libre: (s: string) => `${s} free on the disk`,
    noCabe: (s: string) =>
      `With these limits NB Sound could take up ${s} more than what is free on the disk. Lower a limit or free up space.`,
    cancelar: 'Cancel'
  }
};
