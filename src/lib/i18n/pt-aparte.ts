/**
 * Textos en portugués de Brasil de las pantallas que se cargan aparte del arranque.
 */
import type { TextosAparte } from './es-aparte';
import type { Forma } from './forma';

const n = (v: number) => v.toLocaleString('pt-BR');
const musicas = (v: number) => (v === 1 ? '1 música' : `${n(v)} músicas`);

export const ptAparte: Forma<TextosAparte> = {
  comun: {
    error: 'Não foi possível completar. Tente novamente.'
  },
  apariencia: {
    idioma: 'Idioma',
    idiomaAyuda: 'Muda toda a interface na hora, sem reiniciar.',
    preseleccionesEq: [
      'Plano',
      'Clássica',
      'Club',
      'Dance',
      'Graves',
      'Graves e agudos',
      'Agudos',
      'Fones de ouvido',
      'Salão grande',
      'Ao vivo',
      'Festa',
      'Pop',
      'Reggae',
      'Rock',
      'Ska',
      'Suave',
      'Soft rock',
      'Techno'
    ],
    nombresTemas: {
      negro_puro: 'Preto Puro (OLED)',
      arcilla_nocturna: 'Argila Noturna',
      arcilla_calida: 'Argila Quente',
      oro_liquido: 'Ouro Líquido',
      oscuro_profundo: 'Abismo Profundo',
      electrico_voltaje: 'Voltagem Azul',
      lima_suave: 'Lima Laser',
      carmesi_nocturno: 'Carmesim Noturno',
      aurora_boreal: 'Aurora Boreal',
      selva_neon: 'Selva Neon',
      crepusculo_violeta: 'Crepúsculo Violeta',
      cobalto_night: 'Cobalto Noturno',
      menta_fresh: 'Menta Fresca',
      carbono: 'Carbono Frio',
      vampiro_royal: 'Vampiro Real',
      ambar_calido: 'Âmbar Quente',
      nieve: 'Neve',
      ocean_profundo: 'Oceano Abissal',
      galaxia: 'Galáxia Púrpura',
      desierto_dorado: 'Deserto Dourado',
      bosque_mistico: 'Floresta Mística',
      fuego_solar: 'Fogo Solar',
      sakura_nocturno: 'Sakura Noturna',
      glacial: 'Geleira Azul',
      terminal_verde: 'Terminal Verde',
      sangre_dragon: 'Sangue de Dragão',
      prisma: 'Prisma',
      lava: 'Lava Negra',
      sepia: 'Sépia Imperial',
      medianoche: 'Meia-Noite Azul',
      esmeralda: 'Esmeralda',
      titanio: 'Titânio',
      coral_sunrise: 'Coral Amanhecer',
      indigo_profundo: 'Índigo Profundo',
      aurora_rosa: 'Aurora Rosa',
      cafe_nocturno: 'Café Noturno',
      obsidiana_neon: 'Obsidiana Neon',
      hielo_oled: 'Gelo OLED',
      blanco_editorial: 'Branco Editorial',
      oro_negro: 'Ouro Negro',
      jade_nocturno: 'Jade Noturno',
      plasma_morado: 'Plasma Roxo',
      amanecer_sintetico: 'Amanhecer Sintético',
      tinta_marina: 'Tinta Marinha',
      frambuesa_dark: 'Framboesa Escura',
      cielo_coral: 'Céu Coral',
      violeta_laser: 'Violeta Laser',
      acero_azul: 'Aço Azul',
      rosa_polar: 'Rosa Polar',
      circuito_verde: 'Circuito Verde',
      ultra_violeta: 'Ultravioleta',
      marfil_grafito: 'Marfim Grafite',
      noche_arcade: 'Noite Arcade',
      neon_citrico: 'Neon Cítrico',
      cosmic_latte: 'Latte Cósmico',
      turquesa_electrica: 'Turquesa Elétrica',
      magma_violeta: 'Magma Violeta',
      bruma_artica: 'Névoa Ártica',
      cobre_oxidado: 'Cobre Oxidado',
      sombra_lima: 'Sombra Lima',
      zafiro_medianoche: 'Safira Meia-Noite',
      rosa_quimera: 'Rosa Quimera',
      grafito_radioactivo: 'Grafite Radioativo'
    } as Record<string, string>
  },
  bienvenida: {
    titulo: 'Sua música, na sua melhor versão',
    subtitulo:
      'O NB Sound organiza sua biblioteca, completa capas e letras e a faz soar como merece. Tudo neste computador.',
    nombre: 'Como devemos chamar você?',
    nombreMarcador: 'Seu nome',
    carpetaTitulo: 'Tenho uma pasta com minhas músicas',
    carpetaTexto: 'O NB Sound importa a pasta inteira e acompanha o que você copiar nela depois.',
    carpetaBoton: 'Escolher a pasta',
    sueltaTitulo: 'Quero adicionar músicas avulsas',
    sueltaTexto: 'Escolha arquivos ou arraste-os para a janela quando quiser.',
    sueltaBoton: 'Adicionar músicas',
    copiaTitulo: 'Venho de outro computador',
    copiaTexto: 'Restaure um backup do NB Sound: biblioteca, playlists e histórico.',
    copiaBoton: 'Restaurar um backup',
    buscando:
      'Procurando músicas na sua pasta. Você pode continuar usando o NB Sound enquanto isso.',
    restaurada: 'Backup restaurado. O NB Sound reinicia para aplicá-lo.'
  },
  dondeVive: {
    titulo: 'Onde sua música mora',
    ayuda:
      'Se você mover a pasta, formatar ou vier de outro computador, o NB Sound a procura sozinho ao abrir. Se não encontrar, pergunta onde ela está.',
    sinCarpeta: 'Você ainda não tem pasta de música: ela é escolhida ao importar.',
    todas: (v: number) => (v === 1 ? 'Sua música está aqui.' : `Suas ${n(v)} músicas estão aqui.`),
    faltan: (faltan: number, total: number) =>
      faltan === 1
        ? `Falta 1 das suas ${n(total)} músicas.`
        : `Faltam ${n(faltan)} das suas ${n(total)} músicas.`,
    perdida: 'Sua música não está nesta pasta. Escolha onde ela está agora.',
    vacia: 'Sua biblioteca ainda não tem músicas.',
    comprobar: 'Verificar',
    elegir: 'Escolher',
    comprobado: 'Verificado: tudo está no lugar.',
    preguntaTitulo: 'Onde está sua música?',
    pregunta: (total: number) =>
      `Suas ${n(total)} músicas estavam nesta pasta, mas não estão mais lá. Se você a moveu, formatou ou veio de outro computador, diga onde ela está agora e tudo continuará como você deixou.`,
    buscar: 'Procurar minha música',
    ahoraNo: 'Agora não',
    recuperada: 'Sua música mudou de lugar e o NB Sound a encontrou sozinho.',
    ninguna: 'Nenhuma das suas músicas está lá. Tente outra pasta.'
  },
  carpetaMusica: {
    cambiar: 'Mudar',
    titulo: 'Mudar a pasta da sua música',
    misma: 'Essa já é a pasta da sua música.',
    yaEsta: (v: number) =>
      v === 1
        ? 'Sua música já está lá: o NB Sound só precisa apontar para a nova pasta.'
        : `Suas ${n(v)} músicas já estão lá: o NB Sound só precisa apontar para a nova pasta.`,
    faltan: (encontradas: number, total: number) =>
      encontradas === 0
        ? `Nenhuma das suas ${n(total)} músicas está lá. O NB Sound pode levá-las: cada uma para a mesma subpasta, e as originais são removidas ao terminar.`
        : `Estão lá ${n(encontradas)} das suas ${n(total)} músicas. O NB Sound pode levar as demais: cada uma para a mesma subpasta.`,
    mover: 'Levar minha música',
    usar: 'Usar esta pasta',
    otra: 'Escolher outra',
    parcial: (encontradas: number, total: number) =>
      `Estão lá ${n(encontradas)} das suas ${n(total)} músicas. As demais não poderão tocar até estarem nessa pasta.`,
    usarSinMover: 'Usar sem mover',
    usarSinMoverAviso:
      'Se você não mover a música, as que não estiverem na nova pasta não poderão tocar até que você as leve.',
    moviendo: (hechas: number, total: number) => `Levando sua música… ${n(hechas)} de ${n(total)}`,
    hecho: 'A pasta da sua música agora é a nova.',
    movidas: (v: number) =>
      v === 1
        ? 'Uma música movida para a nova pasta.'
        : `${n(v)} músicas movidas para a nova pasta.`,
    fallo:
      'Não foi possível levar sua música (espaço, permissões?). Tudo voltou ao lugar e a pasta não mudou.',
    dentro: 'A nova pasta não pode estar dentro da atual, nem o contrário.'
  },
  descubrir: {
    titulo: (que: string) => `Para o que você pediu: ${que}`,
    sinAnalisis:
      'Ainda não há músicas analisadas. Elas são analisadas sozinhas em segundo plano depois de importadas.',
    nada: 'Nenhuma música combina o suficiente com o que você pediu.',
    intenciones: {
      triste: 'triste',
      alegre: 'alegre',
      fiesta: 'para festa',
      entrenar: 'para treinar',
      concentrarse: 'para se concentrar',
      tranquila: 'tranquila',
      nocturna: 'noturna',
      intensa: 'intensa',
      suave: 'suave',
      rapida: 'rápida',
      lenta: 'lenta'
    },
    secciones: {
      masFiesteras: 'Mais festeiras',
      masBailables: 'Mais dançantes',
      masRapidas: 'Mais rápidas',
      energiaAlta: 'Energia alta',
      masMelancolicas: 'Mais melancólicas',
      tristesSuaves: 'Tristes mas suaves',
      tristesConEnergia: 'Tristes com mais energia',
      masIntensas: 'Mais intensas',
      ritmoAlto: 'Ritmo alto',
      energiaParaMoverse: 'Energia para se mexer',
      masEnfocadas: 'Mais focadas',
      tranquilasParaTrabajar: 'Tranquilas para trabalhar',
      bajaDistraccion: 'Pouca distração',
      masNocturnas: 'Mais noturnas',
      oscurasYCalmadas: 'Escuras e calmas',
      suavesDeNoche: 'Suaves à noite',
      masTranquilas: 'Mais tranquilas',
      suavesParaBajarRitmo: 'Suaves para desacelerar',
      paraLevantarAnimo: 'Para levantar o ânimo',
      alegresYLigeras: 'Alegres e leves',
      masSuaves: 'Mais suaves',
      bajaEnergia: 'Pouca energia',
      masLentas: 'Mais lentas',
      bajaVelocidad: 'Andamento baixo',
      seleccionSugerida: 'Seleção sugerida'
    } as Record<string, string>
  },
  agregados: {
    ayuda:
      'Funções grandes e opcionais. São baixadas das publicações do NB Sound no GitHub com um clique, e removê-las libera o espaço na hora.',
    nombres: {
      karaoke: 'Karaokê',
      analisis: 'Análise musical',
      textoDj: 'Pedidos ao DJ com suas palavras',
      aceleracion: 'Aceleração por GPU'
    },
    descripciones: {
      karaoke:
        'Separa a voz de qualquer música para cantá-la e prepara os instrumentais para o celular e o DJ.',
      analisis:
        'Reconhece o humor, a dançabilidade e o estilo de cada música para as playlists inteligentes e o DJ.',
      textoDj: 'Entende o que você pede ao DJ com suas próprias palavras.',
      aceleracion:
        'Deixa o karaokê e a análise várias vezes mais rápidos com a placa de vídeo NVIDIA. Vale ao abrir o NB Sound de novo.'
    },
    tamano: (megas: number) => `${megas} MB`,
    instalar: (megas: number) => `Instalar (${megas} MB)`,
    descargando: (porcentaje: number) => `Baixando ${porcentaje}%`,
    instalado: 'Instalado',
    actualizar: 'Atualizar',
    quitar: 'Remover',
    noDisponible: 'Não disponível neste computador',
    ofertaNecesita: (megas: number) =>
      `Precisa de um complemento de ${megas} MB. É baixado uma vez e você pode continuar ouvindo enquanto isso.`,
    ofertaSigue: 'você pode continuar ouvindo',
    ofertaLista: 'Instalado. Toque no microfone para preparar esta música.',
    cerrar: 'Fechar',
    recomendado: (gpu: string) => `Recomendado: seu computador tem ${gpu}.`,
    gpus: {
      cuda: 'uma placa NVIDIA'
    },
    sinConexion: 'Ative os serviços on-line em Privacidade para baixá-lo.',
    errores: {
      NETWORK_FAILURE:
        'O download foi interrompido. Ao tocar em Instalar de novo, ele continua de onde parou.',
      INTEGRITY_MISMATCH: 'O arquivo baixado não é o esperado e foi descartado. Tente novamente.',
      STORAGE_FAILURE: 'Não há espaço ou não foi possível gravar no disco.',
      NETWORK_DISABLED: 'Ative os serviços on-line em Privacidade para baixá-lo.'
    } as Record<string, string>,
    error: 'Não foi possível instalar. Tente novamente.'
  },
  actualizacion: {
    buscar: 'Procurar atualizações',
    buscando: 'Procurando',
    alDia: 'Você tem a versão mais recente.',
    disponible: (v: string) => `O NB Sound ${v} está disponível.`,
    instalar: 'Instalar e reiniciar',
    instalando: 'Baixando e instalando. O NB Sound vai reiniciar sozinho.',
    sinConexion: 'Ative os serviços on-line em Privacidade para procurar atualizações.',
    noDisponible:
      'Esta instalação é atualizada pelo gerenciador de pacotes do sistema ou baixando a nova versão.',
    firma: 'O download não traz a assinatura do NB Sound, então não foi instalado.',
    administrada: 'Esta instalação é atualizada pelo NB Launcher.',
    error: 'Não foi possível conectar. Tente novamente mais tarde.'
  },
  copia: {
    titulo: 'Backup',
    ayuda:
      'Guarda sua biblioteca, playlists, histórico, capas e sua foto em um único arquivo. Não inclui a música nem suas chaves.',
    crear: 'Criar um backup agora',
    creando: 'Criando o backup',
    creada: (megas: string, portadas: number) =>
      `Backup salvo: ${megas} MB, com ${portadas === 1 ? '1 capa' : `${portadas} capas`}.`,
    restaurar: 'Restaurar um backup',
    confirmarTitulo: 'Restaurar um backup?',
    confirmarTexto:
      'Sua biblioteca atual será substituída pela do backup ao reiniciar o NB Sound. A atual fica guardada, caso seja preciso voltar. Sua música não é tocada.',
    elegirCopia: 'Escolher o backup',
    cancelar: 'Cancelar',
    pendiente: 'O backup está pronto. Será aplicado ao reiniciar o NB Sound.',
    reiniciar: 'Reiniciar agora',
    errorFormato: 'Este arquivo não é um backup desta versão do NB Sound.',
    errorNueva:
      'O backup é de uma versão mais nova do NB Sound. Atualize o aplicativo para restaurá-lo.',
    errorDanada: 'O backup está danificado ou incompleto.',
    error: 'Não foi possível completar. Tente novamente.',
    automaticas: 'Backups automáticos',
    conflicto:
      'A frequência foi alterada em outro controle. Mostramos o valor salvo; escolha de novo se quiser mudar.',
    frecuencias: { 0: 'Nunca', 1: 'Todo dia', 7: 'Toda semana', 30: 'Todo mês' } as Record<
      number,
      string
    >,
    carpeta: 'Pasta dos backups',
    sinCarpeta: 'Escolha uma pasta para que sejam feitos.',
    elegirCarpeta: 'Escolher pasta',
    ultima: (fecha: string) => `Último backup: ${fecha}`,
    ninguna: 'Nenhum backup foi feito ainda.'
  },
  edicion: {
    tituloPista: 'Editar dados da música',
    tituloAlbum: 'Editar dados do álbum',
    ayuda:
      'São salvos na sua biblioteca e, se possível, no arquivo. As playlists e o celular continuam reconhecendo a música.',
    titulo: 'Título',
    artista: 'Artista',
    album: 'Álbum',
    albumAyuda: 'Vazio se não pertence a nenhum álbum.',
    numero: 'Número da faixa',
    anio: 'Ano',
    genero: 'Gênero',
    guardar: 'Salvar',
    cancelar: 'Cancelar',
    invalido:
      'Revise os dados: título e artista não podem ficar vazios, o número da faixa vai de 1 a 999 e o ano de 1000 a 9999.',
    sinArchivo: 'O arquivo desta música não foi encontrado.',
    fallo: 'Não foi possível salvar. Tente novamente.'
  },
  privacidad: {
    privacidadAyuda:
      'O NB Sound funciona sem conexão. Cada serviço só é usado se você o ativar, e só recebe o que está indicado.',
    conexion: 'Usar serviços on-line',
    conexionAyuda: 'Desativado, nada sai deste computador, seja qual for o ajuste de cada serviço.',
    servicios: {
      acoustId: {
        nombre: 'AcoustID',
        envia:
          'A impressão acústica de cada música e sua duração, para reconhecê-la. Nunca o áudio.'
      },
      musicBrainz: {
        nombre: 'MusicBrainz',
        envia:
          'Título, artista e álbum, ou o identificador que o AcoustID devolveu, para completar os dados.'
      },
      coverArtArchive: {
        nombre: 'Cover Art Archive',
        envia: 'O identificador do álbum no MusicBrainz, para baixar a capa.'
      },
      itunes: {
        nombre: 'iTunes',
        envia: 'Título e artista, para confirmar a música e procurar capas.'
      },
      lrclib: {
        nombre: 'LRCLIB',
        envia: 'Título, artista, álbum e duração, para procurar a letra sincronizada.'
      },
      lyricsOvh: {
        nombre: 'lyrics.ovh',
        envia: 'Título e artista, para procurar a letra se o LRCLIB não tiver.'
      },
      deezer: {
        nombre: 'Deezer',
        envia: 'O nome do artista ou do álbum, para procurar imagens.'
      },
      wikipedia: {
        nombre: 'Wikipédia',
        envia: 'O nome do artista, para procurar a foto dele.'
      },
      theAudioDb: {
        nombre: 'TheAudioDB',
        envia: 'O nome do artista e do álbum, para procurar capas e imagens do artista.'
      },
      actualizaciones: {
        nombre: 'Atualizações do NB Sound',
        envia:
          'A versão instalada e o sistema, ao GitHub, só quando você procura uma versão nova em Sobre.'
      }
    },
    iaAyuda:
      'Opcional. Quando duas versões de uma música empatam, um modelo de linguagem escolhe entre elas usando sua própria conta. Só recebe os dados da música e dos candidatos, nunca o áudio.',
    iaActiva: 'Desempatar com IA',
    iaSinConexion: 'Ative antes os serviços on-line.',
    proveedor: 'Provedor',
    proveedores: { anthropic: 'Anthropic (Claude)', openai: 'OpenAI' },
    modelo: 'Modelo',
    modeloAyuda: 'Deixe vazio para usar o padrão.',
    clave: 'Chave de API',
    claveGuardada: 'Salva no chaveiro do sistema.',
    claveNueva: 'Cole sua chave aqui',
    guardarClave: 'Salvar chave',
    olvidarClave: 'Esquecer chave',
    sinLlavero:
      'Este sistema não tem chaveiro de senhas, então nenhuma chave pode ser salva. O NB Sound nunca a guarda em um arquivo.',
    noGuardado: 'Não foi possível salvar. Tente novamente.',
    claveServicio: {
      acoustId:
        'Sem ela, uma música não é reconhecida pelo som. É gratuita: peça em acoustid.org com uma conta.'
    }
  },
  inteligentes: {
    nueva: 'Nova inteligente',
    editar: 'Editar regra',
    crearParaMi: 'Criar para mim',
    creadasParaTi: (v: number) =>
      v === 0
        ? 'Suas playlists já estavam em dia com sua música'
        : v === 1
          ? '1 playlist nova ou atualizada com sua música'
          : `${n(v)} playlists novas ou atualizadas com sua música`,
    creandoParaMi: 'Preparando playlists',
    ayuda:
      'Atualiza-se sozinha sempre que você importa música, marca favoritas ou ouve músicas que cumprem estas condições.',
    animo: 'Humor',
    cualquierAnimo: 'Qualquer',
    animos: {
      energetica: 'Energética',
      tranquila: 'Tranquila',
      bailable: 'Dançante',
      triste: 'Triste',
      nocturna: 'Noturna',
      entrenamiento: 'Para treinar',
      concentracion: 'Para se concentrar',
      fiesta: 'Para festa',
      feliz: 'Alegre',
      relajada: 'Relaxada',
      intensa: 'Intensa'
    } as Record<string, string>,
    bpmMin: 'BPM mínimo',
    bpmMax: 'BPM máximo',
    energiaMin: 'Energia mínima (0 a 100%)',
    energiaMax: 'Energia máxima (0 a 100%)',
    instrumental: 'Instrumental de karaokê',
    instrumentalOpciones: {
      cualquiera: 'Qualquer',
      con: 'Só com instrumental pronto',
      sin: 'Só sem instrumental'
    },
    soloFavoritas: 'Só músicas favoritas',
    reproduccionesMin: 'Mínimo de reproduções',
    reproduccionesMax: 'Máximo de reproduções',
    frecuencia: 'Última escuta',
    frecuenciaOpciones: {
      cualquiera: 'A qualquer momento',
      reciente: 'Ouvidas nos últimos dias',
      olvidada: 'Sem ouvir há dias'
    },
    dias: 'Dias',
    anioMin: 'Ano de',
    anioMax: 'Ano até',
    genero: 'Gênero contém',
    orden: 'Ordenar por',
    ordenes: {
      recientes: 'Adicionadas mais recentes',
      masEscuchadas: 'Mais ouvidas',
      menosEscuchadas: 'Menos ouvidas',
      ultimaEscuchaAsc: 'Há mais tempo sem tocar',
      bpmAsc: 'BPM do menor ao maior',
      bpmDesc: 'BPM do maior ao menor',
      energiaDesc: 'Maior energia',
      tituloAsc: 'Título (A a Z)'
    },
    limite: 'Máximo de músicas',
    invalida: 'Revise os intervalos: o mínimo não pode passar do máximo.'
  },

  resumenEscucha: {
    titulo: 'Resumo local de escuta',
    sub: 'Sua atividade por mês, por ano ou em todo o histórico, sem sair do computador.',
    todo: 'Todo o histórico',
    exportar: 'Exportar cartão SVG',
    exportando: 'Exportando…',
    exportada: 'Cartão salvo.',
    radarTitulo: 'Radar de humores',
    radarSub: 'Média das músicas ouvidas no período.',
    ejes: {
      energia: 'Energia',
      bailable: 'Dançante',
      alegre: 'Alegre',
      calma: 'Calma',
      nocturna: 'Noturna',
      melancolica: 'Melancólica'
    },
    diasSemana: ['Seg', 'Ter', 'Qua', 'Qui', 'Sex', 'Sáb', 'Dom'],
    mesesTitulo: 'Mês a mês',
    mesesSub: 'Quanta música você ouviu em cada mês. Toque em um para vê-lo dia a dia.',
    celdaMes: (mes: string, escuchas: number) =>
      `${mes} · ${escuchas} ${escuchas === 1 ? 'escuta' : 'escutas'}`,
    calendarioTitulo: 'Seu mês, dia a dia',
    calendarioSub: 'Quanta música você ouviu em cada dia do mês.',
    celdaDia: (dia: number, escuchas: number) =>
      `Dia ${dia} · ${escuchas} ${escuchas === 1 ? 'escuta' : 'escutas'}`,
    reproduccionesPeriodo: (v: number) => `${n(v)} ${v === 1 ? 'reprodução' : 'reproduções'}`,
    meses: {
      '01': 'Jan',
      '02': 'Fev',
      '03': 'Mar',
      '04': 'Abr',
      '05': 'Mai',
      '06': 'Jun',
      '07': 'Jul',
      '08': 'Ago',
      '09': 'Set',
      '10': 'Out',
      '11': 'Nov',
      '12': 'Dez'
    } as Record<string, string>
  },

  copiaDispositivo: {
    titulo: 'Copiar para um cartão ou pendrive',
    ayuda:
      'Exporta sua música com nomes válidos para FAT32/exFAT, capa incorporada e arquivo de lista .m3u8.',
    origen: 'O que copiar',
    origenes: {
      biblioteca: 'A biblioteca inteira',
      favoritas: 'Minhas músicas favoritas',
      playlist: 'Uma playlist'
    },
    playlist: 'Playlist a copiar',
    sinPlaylists: 'Ainda não há playlists criadas.',
    estructura: 'Estrutura de pastas',
    estructuras: {
      artistaAlbum: 'Artista / Álbum / 01 - Música',
      artista: 'Artista / 01 - Música',
      plana: 'Tudo em uma pasta (Artista - Música)'
    },
    incrustarPortada: 'Incorporar capa e etiquetas nas cópias',
    convertirWav: 'Converter para WAV PCM (16 bits, 44,1 kHz) para máxima compatibilidade',
    botonCopiar: 'Escolher pasta e copiar',
    copiando: 'Copiando para o dispositivo…',
    resumen: (copiadas: number, omitidas: number, mb: string, m3u: string | null) =>
      `${n(copiadas)} ${copiadas === 1 ? 'faixa copiada' : 'faixas copiadas'}${omitidas > 0 ? `, ${n(omitidas)} já existentes` : ''} (${mb} MB)${m3u ? ` · ${m3u}` : ''}.`,
    error: 'Não foi possível completar a cópia para o dispositivo.'
  },

  modoFiesta: {
    titulo: 'Modo Festa',
    ayuda:
      'Seus convidados escaneiam um código e sugerem músicas pelo navegador, sem instalar nada.',
    interruptor: 'Ativar o Modo Festa',
    interruptorAyuda:
      'Só funciona enquanto está ativado. Ao desativar, a porta é fechada e as sessões são apagadas.',
    qrEtiqueta: 'Código QR para entrar no Modo Festa',
    urlEtiqueta: 'Link para convidados na mesma rede:',
    regenerarQr: 'Trocar QR',
    automatico: 'Pôr sugestões na fila automaticamente',
    automaticoAyuda:
      'Se estiver desativado, você aprova ou descarta cada sugestão antes que ela vá para a fila.',
    sugerenciasTitulo: 'Sugestões dos convidados',
    sinSugerencias: 'Ainda não enviaram sugestões.',
    porInvitado: (alias: string) => `Sugerida por ${alias}`,
    enCola: 'Na fila',
    aceptar: 'Aceitar',
    descartar: 'Descartar',
    bloquear: 'Bloquear convidado',
    bloqueados: (v: number) =>
      `${v} ${v === 1 ? 'convidado bloqueado' : 'convidados bloqueados'} nesta sessão`
  },

  duplicados: {
    titulo: 'Músicas duplicadas',
    descripcion:
      'Cópias que o NB Sound separou porque você já tinha a música. Não aparecem nem vão para o celular, e o arquivo continua no lugar.',
    sinDuplicados: 'Não há cópias separadas.',
    encontrados: (v: number, megas: number) =>
      v === 1 ? `1 cópia separada (${n(megas)} MB)` : `${n(v)} cópias separadas (${n(megas)} MB)`,
    limpiar: 'Limpar duplicadas',
    buscar: 'Procurar duplicadas',
    buscando: 'Procurando cópias…',
    apartadas: (v: number) =>
      v === 0
        ? 'Não há cópias novas na sua biblioteca.'
        : v === 1
          ? '1 cópia separada: já não aparece na sua biblioteca.'
          : `${n(v)} cópias separadas: já não aparecem na sua biblioteca.`,
    confirmarTitulo: 'Limpar músicas duplicadas?',
    confirmarPregunta: (v: number) =>
      v === 1
        ? '1 cópia será removida da biblioteca. A música original não é tocada.'
        : `${n(v)} cópias serão removidas da biblioteca. A música original não é tocada.`,
    borrarArchivos: 'Mover também os arquivos para a lixeira',
    purgarBtn: 'Limpar',
    cancelar: 'Cancelar',
    purgadas: (v: number) =>
      v === 1 ? '1 cópia removida da biblioteca' : `${n(v)} cópias removidas da biblioteca`,
    error: 'Não foi possível limpar as cópias.'
  },

  buscar: {
    bienvenida: {
      titulo: 'Peça à sua biblioteca',
      descripcion:
        'Digite um título, um artista, um trecho da letra ou o que você está com vontade de ouvir. Experimente uma destas:'
    },
    ideas: [
      'Músicas alegres para começar o dia',
      'Algo tranquilo para cozinhar',
      'Música para me concentrar',
      'Faixas enérgicas para a academia',
      'Algo triste para uma noite de chuva',
      'Música para festa',
      'Músicas suaves para ler',
      'Para dirigir à noite',
      'Algo chill para relaxar',
      'Faixas intensas e potentes',
      'Dançantes para limpar a casa',
      'Músicas lentas e românticas',
      'Música escura e noturna',
      'Rápidas para correr',
      'Nostalgia para a tarde',
      'Para estudar sem distrações'
    ]
  },
  karaokeConsola: {
    titulo: 'Karaokê',
    subtitulo:
      'Cante qualquer música da sua biblioteca. A voz é separada neste computador, sem internet.',
    abrir: 'Abrir o console de karaokê',
    motorNoInstalado: 'Falta o complemento de karaokê',
    motorExplicacion: 'É o que tira a voz das músicas. É baixado uma vez e funciona sem conexão.',
    instalar: 'Instalar o complemento',
    ahora: 'Tocando agora',
    nadaSonando: 'Toque uma música para cantá-la.',
    lista: 'Pronta para cantar',
    noLista: 'Não preparada',
    preparandoEsta: (pct: number) => `Preparando · ${pct}%`,
    cantar: 'Cantar agora',
    dejarDeCantar: 'Voltar à original',
    prepararPistaActual: 'Preparar esta música',
    enPreparacion: 'Em preparação',
    enEspera: 'Na espera',
    fallidas: (v: number) =>
      v === 1
        ? '1 música não pôde ser preparada. "Preparar minhas favoritas" tenta de novo se ela for favorita.'
        : `${v} músicas não puderam ser preparadas. "Preparar minhas favoritas" tenta de novo com as favoritas.`,
    yMas: (v: number) => (v === 1 ? 'E mais 1 depois.' : `E mais ${v} depois.`),
    siguePorSuCuenta:
      'São preparadas uma após a outra enquanto você ouve. Se você fechar o NB Sound, ele continua de onde parou ao abrir de novo.',
    sinPendientes: 'Nada em preparação. Você pode continuar ouvindo enquanto são preparadas.',
    repertorio: 'Seu repertório',
    listas: (v: number) =>
      v === 1 ? '1 música pronta para cantar' : `${n(v)} músicas prontas para cantar`,
    verListas: 'Ver na biblioteca',
    prepararFavoritas: 'Preparar minhas favoritas',
    prepararPlaylist: 'Preparar uma playlist',
    elegirPlaylist: 'Playlist a preparar',
    playlistEnCola: (v: number) =>
      v === 0
        ? 'Essa playlist já está pronta ou a caminho.'
        : `${musicas(v)} em preparação. Você pode continuar ouvindo.`
  },
  almacenamiento: {
    titulo: 'Armazenamento',
    subtitulo:
      'O que o NB Sound guarda neste computador. O que pode ser refeito tem seu próprio limite: encher uma categoria nunca apaga outra.',
    categorias: {
      portadas: 'Capas e fotos de artistas',
      miniaturas: 'Miniaturas de capas',
      separaciones: 'Separações de voz e instrumentos',
      karaoke: 'Instrumentais do karaokê',
      mashups: 'Mashups criados',
      base_de_datos: 'Biblioteca, histórico e configurações'
    },
    ayudas: {
      portadas:
        'As originais. Não são apagadas: algumas vieram de serviços que talvez não as forneçam mais.',
      miniaturas: 'São refeitas sozinhas ao ver cada capa.',
      separaciones:
        'A voz, a bateria, o baixo e o resto de cada música, para o karaokê e os mashups. Com cada música separada, o próximo mashup ou karaokê com ela leva segundos.',
      karaoke: 'O que toca ao cantar e o que o celular baixa.',
      mashups: 'Os que você criou. Só são apagados se você os apagar.',
      base_de_datos:
        'Sua biblioteca, playlists, histórico e configurações, com os backups de cada atualização.'
    },
    usado: 'Ocupa',
    todaLaBiblioteca: (s: string) => `Com toda a sua biblioteca: cerca de ${s}`,
    limite: 'Limite',
    liberar: 'Liberar',
    liberado: (s: string) => `${s} liberados.`,
    confirmarLiberar: (cat: string, s: string) => `Esvaziar ${cat}? Serão liberados ${s}.`,
    rehacer: (t: string) => `Refazer levaria cerca de ${t} de trabalho do computador.`,
    rehacerSolo: 'São refeitas sozinhas ao usar.',
    total: (s: string) => `O NB Sound ocupa ${s} no total`,
    libre: (s: string) => `${s} livres no disco`,
    noCabe: (s: string) =>
      `Com estes limites, o NB Sound poderia ocupar ${s} a mais do que o espaço livre no disco. Diminua algum limite ou libere espaço.`,
    cancelar: 'Cancelar'
  }
};
