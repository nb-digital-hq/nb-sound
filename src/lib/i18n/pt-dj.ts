/**
 * Textos en portugués de Brasil del DJ Privado y de DJ Mashups.
 */
import type { Tecnica } from '#lib/bindings/index.ts';
import type { TextosDj } from './es-dj';
import type { Forma } from './forma';

export const ptDj: Forma<TextosDj> = {
  titulo: 'DJ Privado',
  subtitulo:
    'Diga o que você quer ouvir e ele monta com sua música uma sessão mixada como um DJ faria.',
  cerrar: 'Fechar (continua no histórico)',
  modos: {
    aMashups: 'DJ Mashups',
    aPrivado: 'DJ Privado',
    irAMashups: 'Mudar para DJ Mashups',
    irAPrivado: 'Mudar para DJ Privado'
  },
  mashups: {
    titulo: 'DJ Mashups',
    subtitulo:
      'Compõe uma peça nova com trechos das suas músicas: vozes de umas sobre as bases de outras, no mesmo andamento e em tonalidades que combinam.',
    origen: 'Músicas',
    toda: 'Toda a sua biblioteca',
    playlist: (nombre: string) => `Playlist: ${nombre}`,
    cuantas: 'Quantas',
    duracion: 'Duração',
    minutos: (n: number) => `${n} min`,
    estilo: 'Estilo',
    estilos: {
      equilibrado: {
        nombre: 'Equilibrado',
        ayuda: 'Frases de oito compassos e uma subida em arco.'
      },
      fiesta: { nombre: 'Festa', ayuda: 'Mais camadas e energia que cresce até o fim.' },
      suave: { nombre: 'Suave', ayuda: 'Poucas camadas e fades longos.' },
      relampago: { nombre: 'Relâmpago', ayuda: 'Cortes rápidos: cabem muito mais músicas.' }
    },
    crear: 'Criar mashup',
    conStems:
      'As vozes são separadas dos instrumentos com o karaokê: na primeira vez leva alguns minutos por música; depois, nada.',
    sinStems:
      'Sem o complemento de karaokê não é possível separar as vozes: o mashup será feito encadeando e fundindo trechos inteiros. Você pode instalá-lo nas Configurações.',
    pocasAnalizadas:
      'São necessárias pelo menos duas músicas com a análise musical feita. O NB Sound as analisa em segundo plano; volte daqui a pouco.',
    ocupado: 'Já há um mashup sendo preparado. Espere terminar ou cancele.',
    error: 'Não foi possível começar o mashup. Tente novamente.',
    etapas: {
      eligiendo: 'Escolhendo músicas compatíveis',
      analizando: 'Analisando o ritmo e as seções',
      separando: 'Separando vozes e instrumentos',
      componiendo: 'Compondo',
      mezclando: 'Mixando o áudio'
    } as Record<string, string>,
    deUnidades: (hechas: number, total: number) => `${hechas} de ${total}`,
    porcentaje: (f: number) => `${Math.round(f * 100)}%`,
    cancelar: 'Cancelar',
    estados: {
      generando: 'Em preparação',
      lista: 'Pronto',
      fallida: 'Não foi possível criar',
      cancelada: 'Cancelado'
    } as Record<string, string>,
    errores: {
      INVALID_INPUT:
        'Não há músicas suficientes que combinem entre si. Tente com mais músicas, outra playlist ou um estilo mais curto.',
      FILE_NOT_FOUND: 'Faltam arquivos de algumas músicas.',
      COMPONENT_UNAVAILABLE: 'O separador de vozes não está disponível.',
      STORAGE_FAILURE: 'Não há espaço no disco ou não foi possível gravar.',
      INTERNAL: 'Foi interrompido antes de terminar.'
    } as Record<string, string>,
    resumen: (canciones: number, duracion: string) =>
      `${canciones === 1 ? '1 música' : `${canciones} músicas`} · ${duracion}`,
    tempo: (bpm: number) => `${Math.round(bpm)} BPM`,
    tonalidad: (nota: string, menor: boolean) => `${nota} ${menor ? 'menor' : 'maior'}`,
    escuchar: 'Ouvir',
    pausar: 'Pausar',
    variante: 'Outra variante',
    exportar: 'Exportar',
    exportado: 'Mashup exportado.',
    noExportable: 'Não é possível exportar dentro da pasta da sua música.',
    eliminar: 'Excluir',
    reintentar: 'Tentar de novo',
    lineaDeTiempo: 'Linha do tempo',
    pistas: 'Músicas do mashup',
    carriles: {
      base: 'Base',
      voz: 'Voz',
      bateria: 'Bateria'
    },
    materiales: {
      mezcla: 'música inteira',
      instrumental: 'instrumental',
      sin_bateria: 'instrumental sem bateria',
      voz: 'voz',
      bateria: 'bateria'
    } as Record<string, string>,
    bloque: (titulo: string, material: string, inicio: string) =>
      `${titulo}, ${material}, a partir de ${inicio}. Ouvir daqui`,
    degradaciones: {
      no_caben: (pedidas: number, caben: number, minimo: string) =>
        `Nessa duração cabem umas ${caben} músicas reconhecíveis; para ${pedidas} seriam precisos uns ${minimo}.`,
      sin_stems: 'Sem separar vozes: trechos inteiros encadeados.',
      menos_canciones: (pedidas: number, usadas: number) =>
        `Você pediu ${pedidas} músicas; ${usadas} combinavam bem.`,
      frases_prolongadas: (n: number) =>
        `${n === 1 ? 'Uma frase prolonga' : `${n} frases prolongam`} a anterior por falta de boas combinações.`,
      mas_corto: (seg: string) => `Dura ${seg}: não havia material para mais sem repetir.`
    }
  },
  pedir: {
    etiqueta: 'O que você quer ouvir?',
    ejemplo: 'Por exemplo: algo tranquilo para trabalhar à noite, sem reggaeton',
    duracion: 'Duração',
    minutos: (n: number) => `${n} min`,
    crear: 'Criar sessão',
    creando: 'Preparando a sessão…',
    error: 'Não foi possível gerar a sessão. Tente novamente.',
    vacia: 'Digite o que você quer ouvir.',
    sinCandidatas:
      'Nenhuma música da sua biblioteca combina com isso. Tente outras palavras ou menos exclusões.',
    ideas: 'Ideias',
    categoriasSugeridas: [
      {
        nombre: 'Vozes',
        chips: ['Vozes femininas', 'Vozes masculinas', 'Instrumental', 'Acústico']
      },
      {
        nombre: 'Gêneros',
        chips: [
          'Reggaeton',
          'Trap',
          'Pop',
          'Salsa',
          'Bachata',
          'Cumbia',
          'Rock',
          'Eletrônica',
          'Hip hop',
          'R&B',
          'Indie',
          'Jazz',
          'Clássica'
        ]
      },
      {
        nombre: 'Humores',
        chips: [
          'Alegre',
          'Romântico',
          'Melancólico',
          'Nostálgico',
          'Noturno',
          'Cinematográfico',
          'Tranquilo'
        ]
      },
      {
        nombre: 'Energia',
        chips: ['Energético', 'Pesado', 'Suave', 'Relaxante', 'Subida progressiva']
      },
      {
        nombre: 'Momentos',
        chips: ['Festa', 'Para treinar', 'Para me concentrar', 'Para cozinhar', 'Noite de chuva']
      }
    ],
    sugerencias: [
      'Subida progressiva para treinar',
      'Algo cinematográfico com vozes femininas',
      'Tranquilo para me concentrar',
      'Festa eletrônica com muita energia',
      'Melancólico para uma noite de chuva'
    ]
  },
  sesion: {
    entendi: 'Entendi',
    evitare: 'Sem',
    conceptos: {
      pop: 'pop',
      rock: 'rock',
      indie: 'indie',
      electronica: 'eletrônica',
      edm_agresivo: 'EDM agressivo',
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
      clasica: 'clássica',
      ambient: 'ambient',
      folk: 'folk',
      metal: 'metal',
      blues: 'blues',
      funk: 'funk',
      cinematografico: 'cinematográfica',
      orquestal: 'orquestral',
      narrativo: 'que conte uma história',
      sonador: 'sonhadora',
      nocturno: 'noturna',
      elegante: 'elegante',
      minimalista: 'minimalista',
      triste: 'triste',
      feliz: 'alegre',
      esperanzador: 'esperançosa',
      tranquilo: 'tranquila',
      agresivo: 'agressiva',
      energetico: 'com energia',
      etereo_calido: 'calorosa',
      voces_femeninas: 'vozes femininas',
      voces_masculinas: 'vozes masculinas',
      instrumental: 'instrumental',
      que_resalten_voces: 'vozes em destaque',
      bajos_fuertes: 'graves fortes',
      acustico: 'acústica',
      electronico_modifier: 'eletrônica',
      oscuro: 'sombria',
      brillante: 'brilhante',
      transiciones_suaves: 'transições suaves',
      transiciones_agresivas: 'transições bruscas',
      transicion_progresiva: 'transição progressiva',
      entrenar: 'para treinar',
      concentrarse: 'para se concentrar',
      club_party: 'festa',
      cocinar: 'para cozinhar',
      lluvioso: 'de chuva',
      lo_fi: 'lo-fi',
      romantico: 'romântica',
      sin_edm_agresivo: 'sem EDM agressivo',
      emocionante: 'emocionante',
      sentimental: 'sentimental',
      melancolico_profundo: 'melancólica',
      nostalgico: 'nostálgica',
      dramatico: 'dramática',
      epico: 'épica',
      que_pegue: 'pesada',
      suave: 'suave',
      relajante: 'relaxante',
      atmosferico: 'atmosférica',
      etereo: 'etérea',
      espacial: 'espacial',
      futurista: 'futurista',
      synthwave: 'synthwave',
      underground: 'underground',
      pesado: 'densa',
      minimal_textural: 'minimal',
      bajos_profundos: 'graves profundos',
      percusion_fuerte: 'percussão forte',
      vocal_dominante: 'a voz como protagonista',
      voz_femenina_suave: 'voz feminina suave',
      voz_masculina_grave: 'voz masculina grave'
    } as Record<string, string>,
    curvas: {
      progressive: 'subida progressiva',
      wave: 'energia em ondas',
      descending: 'energia que desce',
      peak: 'pico de energia'
    } as Record<string, string>,
    general: 'Não reconheci nada concreto: é uma seleção geral da sua música.',
    sonando: 'Tocando',
    enPausa: 'Em pausa',
    construyendo: 'Preparando mais músicas enquanto toca…',
    resumen: (pistas: number, duracion: string, artistas: number) =>
      `${pistas === 1 ? '1 música' : `${pistas} músicas`} · ${duracion} · ${artistas === 1 ? '1 artista' : `${artistas} artistas`}`,
    transiciones: (buenas: number, total: number) =>
      `${buenas} de ${total} transições muito limpas`,
    sinAnalisis:
      'Sem a análise musical, o DJ escolhe com as características básicas de cada música. Com o complemento de análise, nas Configurações, fica mais preciso.',
    escuchar: 'Ouvir',
    regenerar: 'Outra parecida',
    guardar: 'Salvar como playlist',
    guardada: 'Salva em Playlists.',
    descartar: 'Descartar',
    meGusta: 'Gostei',
    noMeGusta: 'Não gostei',
    valorada: 'Vou levar em conta.',
    pistas: 'Músicas da sessão',
    tocarDesde: (titulo: string) => `Ouvir a partir de ${titulo}`,
    entrada: (tiempo: string) => `Entrada ${tiempo}`,
    salida: (tiempo: string) => `Saída ${tiempo}`,
    mezclando: (tecnica: string, progreso: number) => `${tecnica} · ${Math.round(progreso * 100)}%`,
    tecnicas: {
      hard_cut: 'Corte na batida',
      energy_blend: 'Fade longo',
      eq_kill_bass: 'Mixando com equalização',
      filter_sweep: 'Varredura de filtros',
      harmonic_mix: 'Fundindo camadas'
    } satisfies Record<Tecnica, string>,
    transicion: {
      limpia: 'Transição limpa',
      aceptable: 'Transição aceitável',
      forzada: 'Transição forçada'
    }
  },
  historial: {
    titulo: 'Histórico',
    buscar: 'Pesquisar no histórico',
    vacio: 'Você ainda não pediu nenhuma sessão nem nenhum mashup.',
    pocas: 'Suas sessões e mashups ficam aqui.',
    adornoTexto: 'Volte a qualquer um para ouvir de novo, ver as músicas, salvar ou exportar.',
    nada: 'Nada corresponde.',
    tipos: { sesion: 'Sessão', mashup: 'Mashup' },
    eliminarMashup: (titulo: string) => `Excluir o mashup “${titulo}”`,
    estados: {
      construyendo: 'Em preparação',
      lista: 'Pronta',
      finalizada: 'Ouvida',
      descartada: 'Descartada',
      error: 'Sem músicas'
    } as Record<string, string>,
    resumen: (pistas: number, duracion: string) => `${pistas} músicas · ${duracion}`,
    ver: 'Ver',
    eliminar: 'Excluir',
    eliminarSesion: (peticion: string) => `Excluir a sessão “${peticion}”`
  }
};
