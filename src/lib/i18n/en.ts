/**
 * Los textos de la interfaz, en inglés.
 *
 * Tipado con la forma del español: una clave que falte aquí no compila.
 */
import type { Textos } from './es';
import type { Forma } from './forma';

const n = (v: number) => v.toLocaleString('en');

export const en: Forma<Textos> = {
  app: {
    nombre: 'NB Sound'
  },
  comun: {
    cerrar: 'Close'
  },
  avisos: {
    region: 'Notifications',
    regionFallos: 'Errors',
    cerrar: 'Dismiss',
    ventanaNoVisible: 'The NB Sound window could not be shown.',
    veces: (v: number) => (v === 1 ? 'once' : `${v} times`)
  },
  nav: {
    principal: 'Main navigation',
    inicio: 'Home',
    biblioteca: 'Library',
    playlists: 'Playlists',
    dj: 'Private DJ',
    aCiegas: 'Blind Test',
    tuMusica: 'Your Music',
    ajustes: 'Settings',
    porRevisar: 'To Review',
    pendientes: (v: number) => (v === 1 ? '1 pending' : `${v} pending`),
    colapsarLateral: 'Collapse sidebar',
    expandirLateral: 'Expand sidebar',
    secciones: {
      explorar: 'Explore',
      tuMusica: 'Your Music',
      experiencias: 'Experiences'
    }
  },
  busqueda: {
    etiqueta: 'Search your music',
    marcador: 'Songs, albums, artists or lyrics',
    ayudaAtajo: 'Search your music (Ctrl+K)',
    borrar: 'Clear search',
    titulo: 'Search',
    subtitulo: 'Find artists, albums, tracks and lyrics across your whole library.',
    artistas: 'Artists',
    albumes: 'Albums',
    pistas: 'Tracks',
    playlists: 'Playlists',
    sinResultados: (q: string) => `Nothing in your library matches “${q}”.`,
    sinResultadosSub: 'Try fewer words, another spelling or a line from the lyrics.',
    resultados: (q: string) => `Results for “${q}”`,
    mejor: 'Top result',
    canciones: 'Songs',
    todas: 'All songs',
    tipoArtista: 'Artist',
    tipoAlbum: 'Album',
    tipoCancion: 'Song',
    escribe: 'Type above to search.',
    cerrarBusqueda: 'Close search'
  },
  reproductor: {
    region: 'Player',
    nadaSonando: 'Nothing playing',
    aCiegas: 'Blind Test',
    adivina: 'Do you know this one?',
    mashup: 'DJ Mashups',
    eligeAlgo: 'Pick a song to start',
    anterior: 'Previous',
    siguiente: 'Next',
    reproducir: 'Play',
    pausar: 'Pause',
    aleatorio: 'Shuffle',
    repetir: 'Repeat',
    marcarFavorita: 'Add to favorites',
    quitarFavorita: 'Remove from favorites',
    karaoke: 'Karaoke mode',
    karaokePreparar: 'Prepare karaoke for this song',
    letras: 'Lyrics',
    cola: 'Queue',
    colaCon: (v: number) => (v === 1 ? 'Queue, 1 song' : `Queue, ${v} songs`),
    volumen: 'Volume',
    silenciar: 'Mute',
    quitarSilencio: 'Unmute',
    expandir: 'Expanded playback',
    pantallaCompleta: 'Full screen (F11)',
    salirPantallaCompleta: 'Exit full screen (F11)',
    mini: 'Mini player',
    cerrarMini: 'Close the mini player',
    volverDeMini: 'Back to the NB Sound window',
    progreso: 'Song progress',
    repeticion: {
      ninguno: 'Repeat',
      todo: 'Repeat queue',
      uno: 'Repeat this song'
    },
    destino: 'Playing on',
    esteEquipo: 'This computer',
    sonandoAqui: 'Playing here',
    controlandoDesde: (nombre: string) => `${nombre} is in control`,
    escucharEnElTelefono: 'To listen on your phone, choose it from NB Sound Mobile.',
    conectarTelefono: 'Connect a phone',
    masOpciones: 'More options',
    anadirAPlaylist: 'Add to a playlist',
    sorprendeme: 'Surprise me'
  },
  biblioteca: {
    titulo: 'Library',
    vistas: { albumes: 'Albums', artistas: 'Artists', pistas: 'Tracks' },
    cuentaPistas: (v: number) => (v === 1 ? '1 track' : `${n(v)} tracks`),
    cuentaAlbumes: (v: number) => (v === 1 ? '1 album' : `${n(v)} albums`),
    cuentaArtistas: (v: number) => (v === 1 ? '1 artist' : `${n(v)} artists`),
    densidad: {
      gridSm: 'Small grid',
      gridMd: 'Medium grid',
      gridLg: 'Large grid',
      lista: 'List'
    },
    buscar: 'Search by track, artist, album or lyrics',
    favoritas: 'Favorites',
    sinVerificar: 'Unverified',
    paraCantar: 'Ready to sing',
    grupos: { todos: 'All', albumes: 'Albums', sencillos: 'Singles and EPs' },
    favoritasEnCola: (v: number) =>
      v === 0
        ? 'Your favorites are already ready or on their way.'
        : `${v === 1 ? '1 favorite' : `${v} favorites`} being prepared. You can keep listening.`,
    orden: 'Sort',
    ordenes: {
      tituloAsc: 'Title A-Z',
      tituloDesc: 'Title Z-A',
      artistaAsc: 'Artist A-Z',
      artistaDesc: 'Artist Z-A',
      albumAsc: 'Album A-Z',
      albumDesc: 'Album Z-A',
      anioDesc: 'Newest year',
      anioAsc: 'Oldest year',
      duracionDesc: 'Longest',
      duracionAsc: 'Shortest',
      recientes: 'Recent',
      antiguas: 'Oldest',
      masReproducidas: 'Most played',
      menosReproducidas: 'Least played',
      relevancia: 'Relevance',
      nombreAsc: 'Name A-Z',
      nombreDesc: 'Name Z-A',
      masPistas: 'Most tracks'
    },
    reproducir: 'Play',
    anadirACola: 'Add to queue',
    reproducirDespues: 'Play next',
    anadidasACola: (v: number) => (v === 1 ? 'Added to queue' : `${n(v)} songs added to queue`),
    sonaraDespues: (v: number) => (v === 1 ? 'Plays next' : `${n(v)} songs will play next`),
    colaFallo: 'The queue could not be changed.',
    favoritaFallo: 'The favorite could not be saved.',
    explorarFallo: 'This song’s file is no longer on the disk.',
    abrirAlbum: 'Open album',
    abrirArtista: 'Open artist',
    marcarFavorita: 'Mark as favorite',
    quitarFavorita: 'Remove from favorites',
    masAcciones: 'More actions',
    sinVerificarInsignia: 'Unverified',
    completarDatos: 'Complete details',
    editarDatos: 'Edit details',
    mostrarEnExplorador: 'Show in folder',
    borrarCancion: 'Delete song',
    confirmarBorrarTitulo: 'Delete song?',
    confirmarBorrarDetalle: (titulo: string) =>
      `“${titulo}” will leave the library, your playlists and your phone. Its file can stay where it is or go to the trash, where it can be recovered.`,
    borrarSoloBiblioteca: 'Remove from library',
    borrarTambienArchivo: 'Remove and move the file to the trash',
    borrada: (titulo: string) => `“${titulo}” is no longer in the library`,
    borrarFallo: 'The song could not be removed. Its file was not touched.',
    cancelar: 'Cancel',
    sonando: 'Now playing',
    completando: 'Looking up its details',
    completado: (r: { identificada: boolean; portada: boolean; letra: boolean }) => {
      const nuevas = [r.portada && 'the cover', r.letra && 'the lyrics'].filter(Boolean);
      const anadidas = nuevas.length === 0 ? '' : `Added ${nuevas.join(' and ')}.`;
      if (r.identificada) return `Identified. ${anadidas}`.trim();
      return anadidas || 'Nothing new was found.';
    },
    completarSinArchivo: 'This song’s file cannot be found.',
    completarFallo: 'It could not be completed. Try again.',
    lista: 'Songs',
    columnas: { titulo: 'Title', album: 'Album', duracion: 'Length' },
    vacia: 'No songs match.',
    vaciaBiblioteca: 'Your library is empty.',
    vaciaTexto: 'Add songs or a whole folder: they will appear here with their covers and lyrics.',
    sinCoincidencias: 'Nothing matches your search.',
    sinCoincidenciasTexto: 'Try fewer words or remove a filter.',
    cargando: 'Loading',
    tipos: { Album: 'Album', Single: 'Single', EP: 'EP', Compilation: 'Compilation' },
    album: {
      volver: 'Back to the library',
      pistas: (v: number, duracion: string) =>
        `${v === 1 ? '1 track' : `${v} tracks`} · ${duracion}`,
      aleatorio: 'Shuffle',
      noEncontrado: 'This album is no longer in the library.',
      masDe: (artista: string) => `More by ${artista}`
    },
    artista: {
      tipo: 'Artist',
      populares: 'Popular',
      mostrarMas: 'Show more',
      mostrarMenos: 'Show less',
      albumes: 'Albums',
      sencillos: 'Singles and EPs',
      canciones: 'All their songs',
      aleatorio: 'Shuffle',
      resumen: (albumes: number, pistas: number) =>
        `${albumes === 1 ? '1 album' : `${albumes} albums`} · ${pistas === 1 ? '1 track' : `${pistas} tracks`}`,
      noEncontrado: 'This artist is no longer in the library.'
    }
  },
  inicio: {
    sorprendemeAyuda: 'Play one of your songs that has not played in a while (D)',
    pistas: 'Tracks',
    artistas: 'Artists',
    albumes: 'Albums',
    retomar: 'Resume playback',
    continuar: 'Continue',
    destacado: 'Album of the day',
    verAlbum: 'View album',
    reproducir: 'Play',
    vuelve: 'Back to your music',
    vuelveSub: 'Pick up songs, albums and playlists from your library.',
    ultima: 'Last played',
    reproducciones: (v: number) => (v === 1 ? '1 play' : `${v} plays`),
    top: 'Your top 10 songs',
    topSub: 'The ones you have repeated the most.',
    podio: {
      lugar1: '1st place',
      lugar2: '2nd place',
      lugar3: '3rd place'
    },
    playlists: 'Your featured playlists',
    playlistsSub: 'Lists you created or played the most.',
    queGustan: 'Albums that keep coming back',
    queGustanSub: 'Records with favorite songs or frequent plays.',
    artistas10: 'Most played artists',
    artistasSub: 'The names that appear most in your history.',
    anadidos: 'Recently added',
    anadidosSub: 'The latest additions to your library.',
    anterior: 'Previous',
    siguiente: 'Next',
    masEscuchados: 'Your 10 most played albums',
    masEscuchadosSub: 'The records that have been played the most.',
    sorprendeme: (titulo: string, artista: string) => `Surprise me: “${titulo}” by ${artista}`
  },
  tuMusica: {
    fotoInvalida: 'That image could not be used. Choose a PNG or a JPG.',
    identidad: 'Your musical identity',
    anonimo: 'Your Music',
    cambiarNombre: 'Change your name',
    nombreMarcador: 'What should I call you',
    resumen: (r: number, d: number, b: number) =>
      `${n(r)} plays · ${n(d)} different tracks out of ${n(b)} in the library.`,
    anioFavorito: (a: number) => `Favorite year: ${a}`,
    horaPico: (h: string) => `Peak hour: ${h}`,
    diasActivos: (v: number) =>
      v === 1 ? '1 active day this month' : `${v} active days this month`,
    esencia: 'Play your essence',
    actualizar: 'Refresh',
    actividad: 'Your activity',
    actividadSub: 'What your local history says about you.',
    escuchadas: 'Tracks played',
    artistas: 'Different artists',
    albumes: 'Different albums',
    tiempo: 'Time listened',
    animo: 'Mood of the day',
    sinEscuchasHoy: 'You have not listened to anything today yet.',
    escuchasHoy: (v: number) => (v === 1 ? '1 track played today.' : `${v} tracks played today.`),
    mes: 'This month’s activity',
    leyenda: { nada: 'No plays', poco: '1–5', medio: '6–15', mucho: '16+' },
    dia: (d: number, v: number) => `Day ${d}: ${v === 1 ? '1 play' : `${v} plays`}`,
    querido: 'What you loved most',
    queridoSub: 'Your tops by history. Albums and artists open their view in the library.',
    topCanciones: 'Top songs',
    topAlbumes: 'Top albums',
    topArtistas: 'Top artists',
    vacio: 'Listen to music and what your history says about you will appear here.'
  },
  cola: {
    titulo: 'Play queue',
    vaciar: 'Clear',
    vacia: 'The queue is empty. Play something or add it from your library.',
    sonando: 'Playing',
    pausada: 'Paused',
    reproducir: (t: string) => `Play ${t}`,
    quitar: (t: string) => `Remove ${t} from the queue`,
    moverConTeclado: 'Alt+Up and Alt+Down to move'
  },
  reproduciendo: {
    titulo: 'Now Playing',
    volver: 'Back',
    karaoke: 'Karaoke: you sing',
    album: (a: string) => `Album · ${a}`,
    anio: (a: number) => `Year · ${a}`,
    duracion: (d: string) => `Length · ${d}`,
    letras: 'Lyrics',
    sinLetra: 'This song has no saved lyrics.',
    nadaSonando: 'Nothing is playing. Pick a song from your library.',
    irA: (t: string) => `Go to ${t}`,
    verLetras: 'Show lyrics',
    verPortada: 'Show cover'
  },
  playlists: {
    titulo: 'Playlists',
    subtitulo: 'Manual lists, favorites and local mixes built from your library.',
    vaciasTitulo: 'No playlists yet',
    vaciasTexto:
      'Create your own, or press “Create for me” once you have music and NB Sound will build a few from what you listen to.',
    nueva: 'New playlist',
    densidad: {
      gridSm: 'Small grid',
      gridMd: 'Medium grid',
      gridLg: 'Large grid',
      lista: 'List'
    },
    buscar: 'Search by name or type',
    cuenta: (v: number) => (v === 1 ? '1 playlist' : `${v} playlists`),
    categoria: 'Category',
    categorias: {
      todo: 'All',
      meGusta: 'Liked',
      creada: 'Created by me',
      inteligente: 'Made for you',
      thisIs: 'This is...',
      sistema: 'System'
    },
    ordenes: {
      recientes: 'Recent',
      antiguas: 'Oldest',
      nombreAsc: 'A → Z',
      nombreDesc: 'Z → A',
      masCanciones: 'Songs ↓',
      menosCanciones: 'Songs ↑'
    },
    resumen: (v: number, duracion: string) =>
      `${v === 1 ? '1 track' : `${v} tracks`} · ${duracion}`,
    abrir: 'Open',
    anclar: 'Pin',
    desanclar: 'Unpin',
    anclada: 'Pinned',
    reproducir: 'Play',
    aleatorio: 'Shuffle',
    renombrar: 'Rename',
    borrar: 'Delete',
    encolar: 'Add to queue',
    duplicar: 'Duplicate',
    duplicada: (nombre: string) => `Copy created: “${nombre}”`,
    vaciar: 'Clear',
    confirmarVaciarTitulo: 'Clear the playlist?',
    confirmarVaciar: (nombre: string) =>
      `All songs will be removed from “${nombre}”. They stay in your library.`,
    vaciada: 'Playlist cleared',
    anadirCanciones: 'Add songs',
    buscarParaAnadir: 'Search your library to add',
    yaEnLista: 'Already in the playlist',
    confirmarBorrar: (nombre: string) => `Delete “${nombre}”? The songs stay in your library.`,
    quitar: 'Remove from playlist',
    subir: 'Move up',
    bajar: 'Move down',
    vacia: 'This playlist has no songs yet.',
    verMasCanciones: (v: number) => `Show ${n(v)} more songs...`,
    noEncontrada: 'This playlist no longer exists.',
    volver: 'Back to Playlists',
    nombre: 'Name',
    descripcion: 'Description (optional)',
    crear: 'Create',
    guardar: 'Save',
    cancelar: 'Cancel',
    anadirA: 'Add to playlist',
    anadida: (nombre: string) => `Added to “${nombre}”`,
    quitada: (nombre: string) => `Removed from “${nombre}”`,
    yaEstaba: (nombre: string) => `Already in “${nombre}”`,
    errores: {
      INVALID_INPUT: 'The name cannot be empty or longer than 120 characters.',
      NAME_TAKEN: 'You already have a playlist with that name.'
    } as Record<string, string>
  },
  ajustes: {
    titulo: 'Settings',
    secciones: 'Settings sections',
    biblioteca: 'Library',
    reproduccion: 'Playback',
    apariencia: 'Appearance',
    almacenamiento: 'Storage',
    acercaDe: 'About',
    producto: 'NB Sound',
    dispositivos: 'Devices',
    dispositivosSub: 'Your music on your phone, on a card or at the party.',
    telefonos: 'Your phones',
    permitirTelefonos: 'Allow your phones to connect',
    permitirAyuda:
      'NB Sound Mobile syncs your music and controls this computer from the same Wi-Fi network.',
    escuchandoEn: (d: string) => `Available at ${d}`,
    conectarTelefono: 'Connect a phone',
    sinTelefonos: 'You have not connected any phone yet.',
    conectadoAhora: 'Connected now',
    controlando: 'Controlling this computer',
    ultimaConexion: (fecha: string) => `Last connected: ${fecha}`,
    sinConexiones: 'Not connected yet',
    quitar: 'Remove',
    quitarTelefono: (nombre: string) => `Remove ${nombre}`,
    quitarAyuda: (nombre: string) =>
      `${nombre} will stop syncing and controlling this computer. To use it again you will have to connect it again.`,
    cancelar: 'Cancel',
    tituloQr: 'Connect a phone',
    instruccionesQr:
      'Open NB Sound Mobile and scan this code. The phone must be on the same Wi-Fi network as this computer.',
    codigoManual: 'If you cannot scan it, type this code:',
    caduca: (s: number) => `Expires in ${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`,
    caducado: 'The code expired.',
    otroCodigo: 'Generate another code',
    cerrar: 'Close',
    qr: 'QR code to connect a phone',
    emparejado: (nombre: string) => `${nombre} is now connected.`,
    sinServidor:
      'The connection for phones could not be opened. Check that this computer is connected to a network.',
    carpeta: 'Your music folder',
    sinCarpeta: 'No folder yet. You choose it when importing music.',
    nombre: 'Your name',
    nombreAyuda: 'Home uses it to greet you.',
    fotoPerfil: 'Profile picture',
    fotoPerfilAyuda: 'Image shown on your profile and personal statistics.',
    cambiarFoto: 'Change image',
    normalizar: 'Even out volume between songs',
    normalizarAyuda: 'Brings every song to the same level (EBU R128). Applies from the next one.',
    ecualizador: 'Equalizer',
    ecualizadorActivo: 'Equalizer on',
    preseleccion: 'Preset',
    personalizado: 'Custom',
    preamp: 'Preamp',
    banda: (hz: string) => `${hz} band`,
    tema: 'Theme',
    temasOscuros: 'Dark themes',
    temasClaros: 'Light themes',
    temasEspeciales: 'High contrast and OLED',
    tipografia: 'Typeface',
    escala: 'Interface size',
    porcentaje: (v: number) => `${v}%`,
    colorPortada: 'Playback, albums, artists and playlists take the color of their cover',
    muestra: 'NB Sound, your music',
    version: (v: string) => `NB Sound ${v}`,
    licencia: 'Free software under the Apache License 2.0. Add-on models keep their own licenses.',
    terceros: 'Includes third-party components under their own licenses (THIRD-PARTY-NOTICES.md).',
    modelos:
      'Add-ons use Meta’s htdemucs (MIT) for karaoke, the Essentia models from the MTG at Universitat Pompeu Fabra (CC BY-NC, non-commercial use) for analysis, and all-MiniLM-L6-v2 (Apache-2.0) for DJ requests.',
    agregados: 'Add-ons',
    privacidad: 'Privacy and online services',
    ia: 'Artificial intelligence'
  },
  importacion: {
    cancelar: 'Cancel the import: what was already added stays',
    soltar: 'Drop to add to your library',
    soltarAyuda: 'Songs or whole folders. Anything that cannot be identified goes to To Review.',
    anadir: 'Add music',
    anadirArchivos: 'Add files',
    anadirCarpeta: 'Add folder',
    catalogando: (v: number) => (v === 1 ? 'Cataloging 1 song' : `Cataloging ${v} songs`),
    deTotal: (hechos: number, total: number) => `${hechos} of ${total}`,
    deTotalCon: (hechos: number, total: number, archivo: string) =>
      `${hechos} of ${total} · ${archivo}`,
    resumen: (anadidas: number, revisar: number) =>
      revisar > 0 ? `${anadidas} added. ${revisar} to review` : `${anadidas} added`,
    sinVerificar: (v: number) => `${v} unverified`,
    verPorRevisar: 'Open To Review',
    cerrar: 'Close',
    subtitulo: 'Import single tracks or folders into your local library',
    archivosDesc: 'Select audio tracks (.mp3, .flac, .m4a, .wav)',
    carpetaDesc: 'Scans a whole directory with its subfolders and artwork',
    arrastrarTitulo: 'Drag and drop',
    arrastrarDesc: 'You can drag files or folders onto the window at any time',
    arrastrarActivo: 'Drop the files here to start importing',
    estadoBiblioteca: 'Library status',
    archivosPorRevisar: (v: number) =>
      v === 1 ? '1 file waiting for review' : `${v} files waiting for review`,
    sinPendientes: 'Everything cataloged with no issues'
  },
  porRevisar: {
    titulo: 'To Review',
    subtitulo:
      'Songs NB Sound could not confirm on its own. They are grouped by what is wrong, with the usual fix at hand.',
    resumen: 'What is wrong',
    vacioTitulo: 'Nothing to review',
    vacio: 'Everything you imported is in your library.',
    cuenta: (v: number) =>
      v === 1 ? '1 song awaits your decision' : `${v} songs await your decision`,
    enCurso: (hechas: number, total: number) => `Working… ${n(hechas)} of ${n(total)}`,
    grupos: {
      elegir: {
        titulo: 'Is this the right version?',
        ayuda: 'We found what they are called, but we are not sure. Confirm the right one.'
      },
      sinConfirmar: {
        titulo: 'Unconfirmed',
        ayuda: 'They carry their own details, but no source confirms them.'
      },
      noEncontradas: {
        titulo: 'Not found',
        ayuda: 'No source knows them. You can import them with what they carry.'
      },
      repetidas: {
        titulo: 'Already in your library',
        ayuda: 'They are copies of songs already in your library.'
      },
      fallidas: {
        titulo: 'Could not be saved',
        ayuda: 'Something failed while writing their details to the file.'
      },
      danadas: {
        titulo: 'Damaged files',
        ayuda: 'They are incomplete or cannot be read.'
      }
    },
    esEsta: 'This one',
    verMas: (v: number) => (v === 1 ? 'Another version' : `${v} other versions`),
    verMenos: 'Fewer versions',
    importar: 'Import',
    importarTalCual: 'Import as is',
    importarGrupo: (v: number) => (v === 1 ? 'Import' : `Import all ${v}`),
    importarTodas: 'Import all as they are',
    descartar: 'Discard',
    descartarGrupo: (v: number) => (v === 1 ? 'Discard' : `Discard all ${v}`),
    descartarTodas: 'Discard all',
    reintentar: 'Retry',
    reintentarGrupo: (v: number) => (v === 1 ? 'Retry' : `Retry all ${v}`),
    buscarDeNuevo: 'Search again',
    masAcciones: 'More actions',
    sinEtiquetas: 'No tags',
    yaEsta: (cual: string) => `Already in as ${cual}`,
    importada: 'Imported unverified.',
    importadas: (v: number) => (v === 1 ? '1 song imported.' : `${v} songs imported.`),
    elegida: 'Imported with its details.',
    descartadas: (v: number) =>
      v === 1 ? 'Moved to the trash.' : `${v} files moved to the trash.`,
    reintentando: 'Searching again.',
    confirmarImportar: (v: number) =>
      v === 1 ? 'Import 1 unverified song?' : `Import ${v} unverified songs?`,
    faltara: 'They will come in with the details they carry. They will be missing:',
    falta: { portada: 'cover', letra: 'lyrics', album: 'album', anio: 'year' } as Record<
      string,
      string
    >,
    nadaFalta: 'They will come in with the details they carry.',
    confirmarDescartar: (v: number) => (v === 1 ? 'Discard 1 file?' : `Discard ${v} files?`),
    papelera: 'They will go to the system trash: you can recover them from there.',
    confirmar: 'Import',
    confirmarDescarte: 'Discard',
    cancelar: 'Cancel',
    error: 'It could not be completed. Try again.'
  },
  permisos: {
    titulo: 'Permissions',
    ayuda:
      'What NB Sound needs from the system. It is checked automatically, and if something is missing it is requested from here through your system’s own dialog.',
    servicios: { telefono: 'Your phone', fiesta: 'Party Mode' },
    para: {
      telefono: 'Lets NB Sound Mobile find this computer and connect on your network.',
      fiesta: 'Lets your guests open the party page from their phones.'
    },
    estados: {
      abierto: 'Allowed',
      bloqueado: 'Blocked by the firewall',
      sinCortafuegos: 'Allowed',
      desconocido: 'Cannot be checked'
    },
    comprobando: 'Checking',
    permitir: 'Allow',
    pidiendo: 'Waiting for the system',
    cortafuegos: (nombre: string) =>
      `System firewall: ${({ ufw: 'ufw', firewalld: 'firewalld', windows: 'Windows Firewall', macos: 'macOS firewall' } as Record<string, string>)[nombre] ?? nombre}.`,
    nadaMas:
      'NB Sound does not use the microphone, the camera or your location, and only reads the music folder you choose.',
    bloqueadoTitulo: 'The firewall is blocking it',
    avisoAlAbrir: 'Your phone cannot connect. Allow it in Settings > Permissions.',
    bloqueadoTelefono:
      'Your phone cannot reach this computer. Allow it and the system will ask you to confirm.',
    bloqueadoFiesta:
      'Your guests’ phones cannot open the page. Allow it and the system will ask you to confirm.'
  },
  karaokeConsola: {
    titulo: 'Karaoke',
    abrir: 'Open the karaoke console'
  }
};
