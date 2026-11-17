/**
 * Los textos de la interfaz, en portugués de Brasil.
 *
 * Tipado con la forma del español: una clave que falte aquí no compila.
 */
import type { Textos } from './es';
import type { Forma } from './forma';

const n = (v: number) => v.toLocaleString('pt-BR');

export const pt: Forma<Textos> = {
  app: {
    nombre: 'NB Sound'
  },
  comun: {
    cerrar: 'Fechar'
  },
  avisos: {
    region: 'Avisos',
    regionFallos: 'Falhas',
    cerrar: 'Fechar o aviso',
    ventanaNoVisible: 'Não foi possível mostrar a janela do NB Sound.',
    veces: (v: number) => (v === 1 ? '1 vez' : `${v} vezes`)
  },
  nav: {
    principal: 'Navegação principal',
    inicio: 'Início',
    biblioteca: 'Biblioteca',
    playlists: 'Playlists',
    dj: 'DJ Privado',
    aCiegas: 'Às Cegas',
    tuMusica: 'Sua Música',
    ajustes: 'Configurações',
    porRevisar: 'Para Revisar',
    pendientes: (v: number) => (v === 1 ? '1 pendente' : `${v} pendentes`),
    colapsarLateral: 'Recolher barra lateral',
    expandirLateral: 'Expandir barra lateral',
    secciones: {
      explorar: 'Explorar',
      tuMusica: 'Sua Música',
      experiencias: 'Experiências'
    }
  },
  busqueda: {
    etiqueta: 'Pesquisar na sua música',
    marcador: 'Músicas, álbuns, artistas ou letras',
    ayudaAtajo: 'Pesquisar na sua música (Ctrl+K)',
    borrar: 'Limpar pesquisa',
    titulo: 'Pesquisar',
    subtitulo: 'Encontre artistas, álbuns, faixas e letras em toda a sua biblioteca.',
    artistas: 'Artistas',
    albumes: 'Álbuns',
    pistas: 'Faixas',
    playlists: 'Playlists',
    sinResultados: (q: string) => `Nada na sua biblioteca corresponde a “${q}”.`,
    sinResultadosSub: 'Tente menos palavras, outra grafia ou um trecho da letra.',
    resultados: (q: string) => `Resultados de “${q}”`,
    mejor: 'Melhor resultado',
    canciones: 'Músicas',
    todas: 'Todas as músicas',
    tipoArtista: 'Artista',
    tipoAlbum: 'Álbum',
    tipoCancion: 'Música',
    escribe: 'Digite acima para pesquisar.',
    cerrarBusqueda: 'Fechar pesquisa'
  },
  reproductor: {
    region: 'Player',
    nadaSonando: 'Nada tocando',
    aCiegas: 'Às Cegas',
    adivina: 'Sabe qual é?',
    mashup: 'DJ Mashups',
    eligeAlgo: 'Escolha uma música para começar',
    anterior: 'Anterior',
    siguiente: 'Próxima',
    reproducir: 'Tocar',
    pausar: 'Pausar',
    aleatorio: 'Aleatório',
    repetir: 'Repetir',
    marcarFavorita: 'Adicionar às favoritas',
    quitarFavorita: 'Remover das favoritas',
    karaoke: 'Modo karaokê',
    karaokePreparar: 'Preparar o karaokê desta música',
    letras: 'Letras',
    cola: 'Fila',
    colaCon: (v: number) => (v === 1 ? 'Fila, 1 música' : `Fila, ${v} músicas`),
    volumen: 'Volume',
    silenciar: 'Silenciar',
    quitarSilencio: 'Ativar som',
    expandir: 'Reprodução ampliada',
    pantallaCompleta: 'Tela cheia (F11)',
    salirPantallaCompleta: 'Sair da tela cheia (F11)',
    mini: 'Miniplayer',
    cerrarMini: 'Fechar o miniplayer',
    volverDeMini: 'Voltar à janela do NB Sound',
    progreso: 'Progresso da música',
    repeticion: {
      ninguno: 'Repetir',
      todo: 'Repetir a fila',
      uno: 'Repetir esta música'
    },
    destino: 'Onde toca',
    esteEquipo: 'Este computador',
    sonandoAqui: 'Tocando aqui',
    controlandoDesde: (nombre: string) => `${nombre} está no controle`,
    escucharEnElTelefono: 'Para ouvir no celular, escolha-o no NB Sound Mobile.',
    conectarTelefono: 'Conectar um celular',
    masOpciones: 'Mais opções',
    anadirAPlaylist: 'Adicionar a uma playlist',
    sorprendeme: 'Surpreenda-me'
  },
  biblioteca: {
    titulo: 'Biblioteca',
    vistas: { albumes: 'Álbuns', artistas: 'Artistas', pistas: 'Faixas' },
    cuentaPistas: (v: number) => (v === 1 ? '1 faixa' : `${n(v)} faixas`),
    cuentaAlbumes: (v: number) => (v === 1 ? '1 álbum' : `${n(v)} álbuns`),
    cuentaArtistas: (v: number) => (v === 1 ? '1 artista' : `${n(v)} artistas`),
    densidad: {
      gridSm: 'Grade pequena',
      gridMd: 'Grade média',
      gridLg: 'Grade grande',
      lista: 'Lista'
    },
    buscar: 'Pesquisar por faixa, artista, álbum ou letra',
    favoritas: 'Favoritas',
    sinVerificar: 'Não verificadas',
    paraCantar: 'Prontas para cantar',
    grupos: { todos: 'Todos', albumes: 'Álbuns', sencillos: 'Singles e EPs' },
    favoritasEnCola: (v: number) =>
      v === 0
        ? 'Suas favoritas já estão prontas ou a caminho.'
        : `${v === 1 ? '1 favorita' : `${v} favoritas`} em preparação. Você pode continuar ouvindo.`,
    orden: 'Ordem',
    ordenes: {
      tituloAsc: 'Título A-Z',
      tituloDesc: 'Título Z-A',
      artistaAsc: 'Artista A-Z',
      artistaDesc: 'Artista Z-A',
      albumAsc: 'Álbum A-Z',
      albumDesc: 'Álbum Z-A',
      anioDesc: 'Ano mais recente',
      anioAsc: 'Ano mais antigo',
      duracionDesc: 'Maior duração',
      duracionAsc: 'Menor duração',
      recientes: 'Recentes',
      antiguas: 'Antigas',
      masReproducidas: 'Mais tocadas',
      menosReproducidas: 'Menos tocadas',
      relevancia: 'Relevância',
      nombreAsc: 'Nome A-Z',
      nombreDesc: 'Nome Z-A',
      masPistas: 'Mais faixas'
    },
    reproducir: 'Tocar',
    anadirACola: 'Adicionar à fila',
    reproducirDespues: 'Tocar em seguida',
    anadidasACola: (v: number) =>
      v === 1 ? 'Adicionada à fila' : `${n(v)} músicas adicionadas à fila`,
    sonaraDespues: (v: number) =>
      v === 1 ? 'Tocará em seguida' : `${n(v)} músicas tocarão em seguida`,
    colaFallo: 'Não foi possível alterar a fila.',
    favoritaFallo: 'Não foi possível salvar a favorita.',
    explorarFallo: 'O arquivo desta música não está mais no disco.',
    abrirAlbum: 'Abrir álbum',
    abrirArtista: 'Abrir artista',
    marcarFavorita: 'Marcar como favorita',
    quitarFavorita: 'Remover das favoritas',
    masAcciones: 'Mais ações',
    sinVerificarInsignia: 'Não verificada',
    completarDatos: 'Completar dados',
    editarDatos: 'Editar dados',
    mostrarEnExplorador: 'Mostrar na pasta',
    borrarCancion: 'Excluir música',
    confirmarBorrarTitulo: 'Excluir música?',
    confirmarBorrarDetalle: (titulo: string) =>
      `“${titulo}” sairá da biblioteca, das suas playlists e do celular. O arquivo pode ficar onde está ou ir para a lixeira, de onde pode ser recuperado.`,
    borrarSoloBiblioteca: 'Remover da biblioteca',
    borrarTambienArchivo: 'Remover e mover o arquivo para a lixeira',
    borrada: (titulo: string) => `“${titulo}” não está mais na biblioteca`,
    borrarFallo: 'Não foi possível remover a música. O arquivo não foi tocado.',
    cancelar: 'Cancelar',
    sonando: 'Tocando agora',
    completando: 'Buscando os dados',
    completado: (r: { identificada: boolean; portada: boolean; letra: boolean }) => {
      const nuevas = [r.portada && 'a capa', r.letra && 'a letra'].filter(Boolean);
      const anadidas =
        nuevas.length === 0
          ? ''
          : `${nuevas.length === 1 ? 'Foi adicionada' : 'Foram adicionadas'} ${nuevas.join(' e ')}.`;
      if (r.identificada) return `Identificada. ${anadidas}`.trim();
      return anadidas || 'Nada novo foi encontrado.';
    },
    completarSinArchivo: 'O arquivo desta música não foi encontrado.',
    completarFallo: 'Não foi possível completar. Tente novamente.',
    lista: 'Músicas',
    columnas: { titulo: 'Título', album: 'Álbum', duracion: 'Duração' },
    vacia: 'Nenhuma música corresponde.',
    vaciaBiblioteca: 'Sua biblioteca está vazia.',
    vaciaTexto: 'Adicione músicas ou uma pasta inteira: elas aparecerão aqui com capas e letras.',
    sinCoincidencias: 'Nada corresponde à sua pesquisa.',
    sinCoincidenciasTexto: 'Tente menos palavras ou remova algum filtro.',
    cargando: 'Carregando',
    tipos: { Album: 'Álbum', Single: 'Single', EP: 'EP', Compilation: 'Coletânea' },
    album: {
      volver: 'Voltar à biblioteca',
      pistas: (v: number, duracion: string) =>
        `${v === 1 ? '1 faixa' : `${v} faixas`} · ${duracion}`,
      aleatorio: 'Aleatório',
      noEncontrado: 'Este álbum não está mais na biblioteca.',
      masDe: (artista: string) => `Mais de ${artista}`
    },
    artista: {
      tipo: 'Artista',
      populares: 'Populares',
      mostrarMas: 'Mostrar mais',
      mostrarMenos: 'Mostrar menos',
      albumes: 'Álbuns',
      sencillos: 'Singles e EPs',
      canciones: 'Todas as músicas',
      aleatorio: 'Aleatório',
      resumen: (albumes: number, pistas: number) =>
        `${albumes === 1 ? '1 álbum' : `${albumes} álbuns`} · ${pistas === 1 ? '1 faixa' : `${pistas} faixas`}`,
      noEncontrado: 'Este artista não está mais na biblioteca.'
    }
  },
  inicio: {
    sorprendemeAyuda: 'Toque uma música sua que não toca há tempos (D)',
    pistas: 'Faixas',
    artistas: 'Artistas',
    albumes: 'Álbuns',
    retomar: 'Retomar reprodução',
    continuar: 'Continuar',
    destacado: 'Álbum do dia',
    verAlbum: 'Ver álbum',
    reproducir: 'Tocar',
    vuelve: 'Volte à sua música',
    vuelveSub: 'Retome músicas, álbuns e playlists da sua biblioteca.',
    ultima: 'Última reprodução',
    reproducciones: (v: number) => (v === 1 ? '1 reprodução' : `${v} reproduções`),
    top: 'Suas 10 músicas mais tocadas',
    topSub: 'As que você mais repetiu.',
    podio: {
      lugar1: '1.º lugar',
      lugar2: '2.º lugar',
      lugar3: '3.º lugar'
    },
    playlists: 'Suas playlists em destaque',
    playlistsSub: 'Listas que você criou ou mais ouviu.',
    queGustan: 'Álbuns que sempre voltam',
    queGustanSub: 'Discos com músicas favoritas ou escutas frequentes.',
    artistas10: 'Artistas que mais tocam',
    artistasSub: 'Os nomes que mais aparecem no seu histórico.',
    anadidos: 'Adicionados recentemente',
    anadidosSub: 'O que entrou por último na sua biblioteca.',
    anterior: 'Anterior',
    siguiente: 'Próximo',
    masEscuchados: 'Seus 10 álbuns mais ouvidos',
    masEscuchadosSub: 'Os discos que mais tocaram.',
    sorprendeme: (titulo: string, artista: string) => `Surpreenda-me: “${titulo}” de ${artista}`
  },
  tuMusica: {
    fotoInvalida: 'Não foi possível usar essa imagem. Escolha um PNG ou um JPG.',
    identidad: 'Sua identidade musical',
    anonimo: 'Sua Música',
    cambiarNombre: 'Mudar seu nome',
    nombreMarcador: 'Como você quer ser chamado',
    resumen: (r: number, d: number, b: number) =>
      `${n(r)} reproduções · ${n(d)} faixas diferentes de ${n(b)} na biblioteca.`,
    anioFavorito: (a: number) => `Ano favorito: ${a}`,
    horaPico: (h: string) => `Horário de pico: ${h}`,
    diasActivos: (v: number) => (v === 1 ? '1 dia ativo este mês' : `${v} dias ativos este mês`),
    esencia: 'Tocar sua essência',
    actualizar: 'Atualizar',
    actividad: 'Sua atividade',
    actividadSub: 'O que seu histórico local diz sobre você.',
    escuchadas: 'Faixas ouvidas',
    artistas: 'Artistas diferentes',
    albumes: 'Álbuns diferentes',
    tiempo: 'Tempo ouvido',
    animo: 'Humor do dia',
    sinEscuchasHoy: 'Você ainda não ouviu nada hoje.',
    escuchasHoy: (v: number) => (v === 1 ? '1 faixa tocada hoje.' : `${v} faixas tocadas hoje.`),
    mes: 'Atividade do mês',
    leyenda: { nada: 'Sem escutas', poco: '1–5', medio: '6–15', mucho: '16+' },
    dia: (d: number, v: number) => `Dia ${d}: ${v === 1 ? '1 escuta' : `${v} escutas`}`,
    querido: 'O que você mais amou',
    queridoSub: 'Seus tops pelo histórico. Álbum ou artista levam à sua página na biblioteca.',
    topCanciones: 'Top músicas',
    topAlbumes: 'Top álbuns',
    topArtistas: 'Top artistas',
    vacio: 'Ouça música e aqui aparecerá o que seu histórico diz sobre você.'
  },
  cola: {
    titulo: 'Fila de reprodução',
    vaciar: 'Esvaziar',
    vacia: 'A fila está vazia. Toque algo ou adicione da sua biblioteca.',
    sonando: 'Tocando',
    pausada: 'Pausada',
    reproducir: (t: string) => `Tocar ${t}`,
    quitar: (t: string) => `Remover ${t} da fila`,
    moverConTeclado: 'Alt+Cima e Alt+Baixo para mover'
  },
  reproduciendo: {
    titulo: 'Tocando Agora',
    volver: 'Voltar',
    karaoke: 'Karaokê: cante você',
    album: (a: string) => `Álbum · ${a}`,
    anio: (a: number) => `Ano · ${a}`,
    duracion: (d: string) => `Duração · ${d}`,
    letras: 'Letras',
    sinLetra: 'Esta música não tem letra salva.',
    nadaSonando: 'Nada está tocando. Escolha uma música na sua biblioteca.',
    irA: (t: string) => `Ir para ${t}`,
    verLetras: 'Ver letras',
    verPortada: 'Ver capa'
  },
  playlists: {
    titulo: 'Playlists',
    subtitulo: 'Listas manuais, favoritas e mixes locais feitos com a sua biblioteca.',
    vaciasTitulo: 'Ainda não há playlists',
    vaciasTexto:
      'Crie a sua, ou toque em “Criar para mim” quando tiver música e o NB Sound montará algumas com o que você ouve.',
    nueva: 'Nova playlist',
    densidad: {
      gridSm: 'Grade pequena',
      gridMd: 'Grade média',
      gridLg: 'Grade grande',
      lista: 'Lista'
    },
    buscar: 'Pesquisar por nome ou tipo',
    cuenta: (v: number) => (v === 1 ? '1 playlist' : `${v} playlists`),
    categoria: 'Categoria',
    categorias: {
      todo: 'Tudo',
      meGusta: 'Curtidas',
      creada: 'Criadas por mim',
      inteligente: 'Feitas para você',
      thisIs: 'This is...',
      sistema: 'Sistema'
    },
    ordenes: {
      recientes: 'Recentes',
      antiguas: 'Antigas',
      nombreAsc: 'A → Z',
      nombreDesc: 'Z → A',
      masCanciones: 'Músicas ↓',
      menosCanciones: 'Músicas ↑'
    },
    resumen: (v: number, duracion: string) =>
      `${v === 1 ? '1 faixa' : `${v} faixas`} · ${duracion}`,
    abrir: 'Abrir',
    anclar: 'Fixar',
    desanclar: 'Desafixar',
    anclada: 'Fixada',
    reproducir: 'Tocar',
    aleatorio: 'Aleatório',
    renombrar: 'Renomear',
    borrar: 'Excluir',
    encolar: 'Adicionar à fila',
    duplicar: 'Duplicar',
    duplicada: (nombre: string) => `Cópia criada: “${nombre}”`,
    vaciar: 'Esvaziar',
    confirmarVaciarTitulo: 'Esvaziar a playlist?',
    confirmarVaciar: (nombre: string) =>
      `Todas as músicas de “${nombre}” serão removidas. Elas continuam na sua biblioteca.`,
    vaciada: 'Playlist esvaziada',
    anadirCanciones: 'Adicionar músicas',
    buscarParaAnadir: 'Pesquise na sua biblioteca para adicionar',
    yaEnLista: 'Já está na playlist',
    confirmarBorrar: (nombre: string) =>
      `Excluir “${nombre}”? As músicas continuam na sua biblioteca.`,
    quitar: 'Remover da playlist',
    subir: 'Subir',
    bajar: 'Descer',
    vacia: 'Esta playlist ainda não tem músicas.',
    verMasCanciones: (v: number) => `Ver mais ${n(v)} músicas...`,
    noEncontrada: 'Esta playlist não existe mais.',
    volver: 'Voltar a Playlists',
    nombre: 'Nome',
    descripcion: 'Descrição (opcional)',
    crear: 'Criar',
    guardar: 'Salvar',
    cancelar: 'Cancelar',
    anadirA: 'Adicionar à playlist',
    anadida: (nombre: string) => `Adicionada a “${nombre}”`,
    quitada: (nombre: string) => `Removida de “${nombre}”`,
    yaEstaba: (nombre: string) => `Já estava em “${nombre}”`,
    errores: {
      INVALID_INPUT: 'O nome não pode ficar vazio nem passar de 120 caracteres.',
      NAME_TAKEN: 'Você já tem uma playlist com esse nome.'
    } as Record<string, string>
  },
  ajustes: {
    titulo: 'Configurações',
    secciones: 'Seções das Configurações',
    biblioteca: 'Biblioteca',
    reproduccion: 'Reprodução',
    apariencia: 'Aparência',
    almacenamiento: 'Armazenamento',
    acercaDe: 'Sobre',
    producto: 'NB Sound',
    dispositivos: 'Dispositivos',
    dispositivosSub: 'Sua música no celular, num cartão ou na festa.',
    telefonos: 'Seus celulares',
    permitirTelefonos: 'Permitir que seus celulares se conectem',
    permitirAyuda:
      'O NB Sound Mobile sincroniza sua música e controla este computador na mesma rede Wi-Fi.',
    escuchandoEn: (d: string) => `Disponível em ${d}`,
    conectarTelefono: 'Conectar um celular',
    sinTelefonos: 'Você ainda não conectou nenhum celular.',
    conectadoAhora: 'Conectado agora',
    controlando: 'Controlando este computador',
    ultimaConexion: (fecha: string) => `Última conexão: ${fecha}`,
    sinConexiones: 'Ainda não se conectou',
    quitar: 'Remover',
    quitarTelefono: (nombre: string) => `Remover ${nombre}`,
    quitarAyuda: (nombre: string) =>
      `${nombre} deixará de sincronizar e de controlar este computador. Para usá-lo de novo, será preciso conectá-lo outra vez.`,
    cancelar: 'Cancelar',
    tituloQr: 'Conectar um celular',
    instruccionesQr:
      'Abra o NB Sound Mobile e escaneie este código. O celular precisa estar na mesma rede Wi-Fi que este computador.',
    codigoManual: 'Se não conseguir escanear, digite este código:',
    caduca: (s: number) => `Expira em ${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`,
    caducado: 'O código expirou.',
    otroCodigo: 'Gerar outro código',
    cerrar: 'Fechar',
    qr: 'Código QR para conectar um celular',
    emparejado: (nombre: string) => `${nombre} já está conectado.`,
    sinServidor:
      'Não foi possível abrir a conexão com os celulares. Verifique se este computador está conectado a uma rede.',
    carpeta: 'Pasta da sua música',
    sinCarpeta: 'Ainda não há pasta. Você a escolhe ao importar música.',
    nombre: 'Seu nome',
    nombreAyuda: 'O Início usa para cumprimentar você.',
    fotoPerfil: 'Foto de perfil',
    fotoPerfilAyuda: 'Imagem exibida no seu perfil e nas estatísticas pessoais.',
    cambiarFoto: 'Mudar imagem',
    normalizar: 'Igualar o volume entre músicas',
    normalizarAyuda: 'Leva todas as músicas ao mesmo nível (EBU R128). Vale a partir da próxima.',
    ecualizador: 'Equalizador',
    ecualizadorActivo: 'Equalizador ativo',
    preseleccion: 'Predefinição',
    personalizado: 'Personalizado',
    preamp: 'Pré-amplificação',
    banda: (hz: string) => `Banda de ${hz}`,
    tema: 'Tema',
    temasOscuros: 'Temas escuros',
    temasClaros: 'Temas claros',
    temasEspeciales: 'Alto contraste e OLED',
    tipografia: 'Tipografia',
    escala: 'Tamanho da interface',
    porcentaje: (v: number) => `${v}%`,
    colorPortada: 'A reprodução, os álbuns, os artistas e as playlists usam a cor da capa',
    muestra: 'NB Sound, sua música',
    version: (v: string) => `NB Sound ${v}`,
    licencia:
      'Software livre sob a licença Apache 2.0. Os modelos dos complementos têm suas próprias licenças.',
    terceros: 'Inclui componentes de terceiros com suas licenças (THIRD-PARTY-NOTICES.md).',
    modelos:
      'Os complementos usam o htdemucs da Meta (MIT) para o karaokê, os modelos do Essentia do MTG da Universitat Pompeu Fabra (CC BY-NC, uso não comercial) para a análise e o all-MiniLM-L6-v2 (Apache-2.0) para os pedidos ao DJ.',
    agregados: 'Complementos',
    privacidad: 'Privacidade e serviços on-line',
    ia: 'Inteligência artificial'
  },
  importacion: {
    cancelar: 'Cancelar a importação: o que já foi adicionado fica',
    soltar: 'Solte para adicionar à sua biblioteca',
    soltarAyuda:
      'Músicas ou pastas inteiras. O que não puder ser identificado fica em Para Revisar.',
    anadir: 'Adicionar música',
    anadirArchivos: 'Adicionar arquivos',
    anadirCarpeta: 'Adicionar pasta',
    catalogando: (v: number) => (v === 1 ? 'Catalogando 1 música' : `Catalogando ${v} músicas`),
    deTotal: (hechos: number, total: number) => `${hechos} de ${total}`,
    deTotalCon: (hechos: number, total: number, archivo: string) =>
      `${hechos} de ${total} · ${archivo}`,
    resumen: (anadidas: number, revisar: number) =>
      revisar > 0
        ? `${anadidas === 1 ? '1 adicionada' : `${anadidas} adicionadas`}. ${revisar} para revisar`
        : anadidas === 1
          ? '1 adicionada'
          : `${anadidas} adicionadas`,
    sinVerificar: (v: number) => (v === 1 ? '1 não verificada' : `${v} não verificadas`),
    verPorRevisar: 'Ver Para Revisar',
    cerrar: 'Fechar',
    subtitulo: 'Importe faixas avulsas ou pastas para a sua biblioteca local',
    archivosDesc: 'Selecione faixas de áudio (.mp3, .flac, .m4a, .wav)',
    carpetaDesc: 'Varre um diretório inteiro com subpastas e capas',
    arrastrarTitulo: 'Arrastar e soltar',
    arrastrarDesc: 'Você pode arrastar arquivos ou pastas para a janela a qualquer momento',
    arrastrarActivo: 'Solte os arquivos aqui para começar a importar',
    estadoBiblioteca: 'Estado da biblioteca',
    archivosPorRevisar: (v: number) =>
      v === 1 ? '1 arquivo pendente para revisar' : `${v} arquivos pendentes para revisar`,
    sinPendientes: 'Todo o conteúdo catalogado sem problemas'
  },
  porRevisar: {
    titulo: 'Para Revisar',
    subtitulo:
      'Músicas que o NB Sound não conseguiu confirmar sozinho. Estão agrupadas pelo problema, com a solução mais comum à mão.',
    resumen: 'O que acontece',
    vacioTitulo: 'Nada para revisar',
    vacio: 'Tudo o que você importou está na sua biblioteca.',
    cuenta: (v: number) =>
      v === 1 ? '1 música aguarda sua decisão' : `${v} músicas aguardam sua decisão`,
    enCurso: (hechas: number, total: number) => `Trabalhando… ${n(hechas)} de ${n(total)}`,
    grupos: {
      elegir: {
        titulo: 'É a versão certa?',
        ayuda: 'Encontramos os nomes, mas não temos certeza. Confirme a correta.'
      },
      sinConfirmar: {
        titulo: 'Sem confirmação',
        ayuda: 'Trazem seus dados, mas nenhuma fonte os confirma.'
      },
      noEncontradas: {
        titulo: 'Não encontradas',
        ayuda: 'Nenhuma fonte as conhece. Você pode importá-las com o que trazem.'
      },
      repetidas: {
        titulo: 'Você já tem',
        ayuda: 'São cópias de músicas que já estão na sua biblioteca.'
      },
      fallidas: {
        titulo: 'Não foi possível salvar',
        ayuda: 'Algo falhou ao gravar os dados no arquivo.'
      },
      danadas: {
        titulo: 'Arquivos danificados',
        ayuda: 'Estão incompletos ou não podem ser lidos.'
      }
    },
    esEsta: 'É esta',
    verMas: (v: number) => (v === 1 ? 'Outra versão' : `Outras ${v} versões`),
    verMenos: 'Menos versões',
    importar: 'Importar',
    importarTalCual: 'Importar como está',
    importarGrupo: (v: number) => (v === 1 ? 'Importar' : `Importar as ${v}`),
    importarTodas: 'Importar todas como estão',
    descartar: 'Descartar',
    descartarGrupo: (v: number) => (v === 1 ? 'Descartar' : `Descartar as ${v}`),
    descartarTodas: 'Descartar todas',
    reintentar: 'Tentar de novo',
    reintentarGrupo: (v: number) => (v === 1 ? 'Tentar de novo' : `Tentar de novo as ${v}`),
    buscarDeNuevo: 'Buscar de novo',
    masAcciones: 'Mais ações',
    sinEtiquetas: 'Sem etiquetas',
    yaEsta: (cual: string) => `Já está como ${cual}`,
    importada: 'Importada sem verificação.',
    importadas: (v: number) => (v === 1 ? '1 música importada.' : `${v} músicas importadas.`),
    elegida: 'Importada com seus dados.',
    descartadas: (v: number) =>
      v === 1 ? 'Enviada para a lixeira.' : `${v} arquivos enviados para a lixeira.`,
    reintentando: 'Buscando de novo.',
    confirmarImportar: (v: number) =>
      v === 1 ? 'Importar 1 música sem verificação?' : `Importar ${v} músicas sem verificação?`,
    faltara: 'Entrarão com os dados que trazem. Vai faltar:',
    falta: { portada: 'capa', letra: 'letra', album: 'álbum', anio: 'ano' } as Record<
      string,
      string
    >,
    nadaFalta: 'Entrarão com os dados que trazem.',
    confirmarDescartar: (v: number) =>
      v === 1 ? 'Descartar 1 arquivo?' : `Descartar ${v} arquivos?`,
    papelera: 'Irão para a lixeira do sistema: você pode recuperá-los de lá.',
    confirmar: 'Importar',
    confirmarDescarte: 'Descartar',
    cancelar: 'Cancelar',
    error: 'Não foi possível completar. Tente novamente.'
  },
  permisos: {
    titulo: 'Permissões',
    ayuda:
      'O que o NB Sound precisa do sistema. É verificado automaticamente e, se faltar algo, é pedido daqui com a janela do seu sistema.',
    servicios: { telefono: 'Seu celular', fiesta: 'O Modo Festa' },
    para: {
      telefono: 'Que o NB Sound Mobile encontre este computador e se conecte na sua rede.',
      fiesta: 'Que seus convidados abram a página da festa pelo celular.'
    },
    estados: {
      abierto: 'Permitido',
      bloqueado: 'Bloqueado pelo firewall',
      sinCortafuegos: 'Permitido',
      desconocido: 'Não é possível verificar'
    },
    comprobando: 'Verificando',
    permitir: 'Permitir',
    pidiendo: 'Aguardando o sistema',
    cortafuegos: (nombre: string) =>
      `Firewall do sistema: ${({ ufw: 'ufw', firewalld: 'firewalld', windows: 'Firewall do Windows', macos: 'firewall do macOS' } as Record<string, string>)[nombre] ?? nombre}.`,
    nadaMas:
      'O NB Sound não usa o microfone, a câmera nem sua localização, e só lê a pasta de música que você escolhe.',
    bloqueadoTitulo: 'O firewall está bloqueando',
    avisoAlAbrir: 'Seu celular não consegue se conectar. Permita em Configurações > Permissões.',
    bloqueadoTelefono:
      'Seu celular não consegue chegar a este computador. Permita e o sistema pedirá sua confirmação.',
    bloqueadoFiesta:
      'Os celulares dos seus convidados não conseguem abrir a página. Permita e o sistema pedirá sua confirmação.'
  },
  karaokeConsola: {
    titulo: 'Karaokê',
    abrir: 'Abrir o console de karaokê'
  }
};
