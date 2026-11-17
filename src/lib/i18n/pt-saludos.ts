/** Los saludos de Inicio, en portugués de Brasil. */

import type { TextosSaludos } from './es-saludos';
import type { Forma } from './forma';

export const ptSaludos: Forma<TextosSaludos> = {
  madrugada: [
    (nombre: string) => `Boa noite, ${nombre}. A música acompanha a calma destas horas.`,
    (nombre: string) => `Madrugada tranquila, ${nombre}. Uma boa seleção para ouvir sem pressa.`,
    (nombre: string) => `Boa noite, ${nombre}. Sua biblioteca continua acordada com você.`,
    (nombre: string) => `${nombre}, ouça com calma neste trecho da noite.`,
    (nombre: string) => `Boa noite, ${nombre}. O melhor momento para uma escuta atenta.`
  ],
  manana: [
    (nombre: string) => `Bom dia, ${nombre}. Sua música já está pronta.`,
    (nombre: string) => `Comece o dia com uma boa seleção, ${nombre}.`,
    (nombre: string) => `Bom dia, ${nombre}. Sua biblioteca estava esperando por você.`,
    (nombre: string) => `Hoje pode começar com algo que você já conhece, ${nombre}.`,
    (nombre: string) => `Bom dia, ${nombre}. Há música esperando para ser retomada.`,
    (nombre: string) => `${nombre}, comece por uma música que marque o ritmo.`,
    (nombre: string) => `A manhã soa melhor quando você escolhe bem, ${nombre}.`,
    (nombre: string) => `Sua biblioteca tem bom material para hoje, ${nombre}.`,
    (nombre: string) => `${nombre}, volte ao que você estava ouvindo.`,
    (nombre: string) => `Bom dia, ${nombre}. Escolha o primeiro tom do dia.`,
    (nombre: string) => `Há uma sessão esperando por você, ${nombre}.`,
    (nombre: string) => `${nombre}, suas favoritas estão prontas para tocar de novo.`,
    (nombre: string) => `Bom dia, ${nombre}. Sua biblioteca está em ordem.`,
    (nombre: string) => `${nombre}, abra a manhã com algo seu.`,
    (nombre: string) => `A primeira música pode mudar o ritmo, ${nombre}.`,
    (nombre: string) => `Bom dia, ${nombre}. Veja o que soa melhor agora.`
  ],
  tarde: [
    (nombre: string) => `Boa tarde, ${nombre}. Sua próxima escuta está perto.`,
    (nombre: string) => `Pausa musical, ${nombre}. Há muito para retomar.`,
    (nombre: string) => `Boa tarde, ${nombre}. Dê play em algo que mexa com você.`,
    (nombre: string) => `Sua tarde pode seguir com boa música, ${nombre}.`,
    (nombre: string) => `Não deixe o ritmo cair, ${nombre}.`,
    (nombre: string) => `${nombre}, uma boa playlist organiza qualquer tarde.`,
    (nombre: string) => `Sua biblioteca tem opções para esta hora, ${nombre}.`,
    (nombre: string) => `Hora perfeita para redescobrir favoritas, ${nombre}.`,
    (nombre: string) => `Boa tarde, ${nombre}. Dê espaço a uma música diferente.`,
    (nombre: string) => `${nombre}, esta tarde pode soar com mais intenção.`,
    (nombre: string) => `${nombre}, volte às músicas que você deixou pela metade.`,
    (nombre: string) => `Sua biblioteca tem exatamente o que você precisa agora, ${nombre}.`,
    (nombre: string) => `Boa tarde, ${nombre}. Há álbuns que merecem outra volta.`,
    (nombre: string) => `${nombre}, escolha algo familiar ou tente outro caminho.`,
    (nombre: string) => `A tarde está pronta para uma sessão curta, ${nombre}.`,
    (nombre: string) => `${nombre}, seus artistas frequentes estão esperando.`
  ],
  noche: [
    (nombre: string) => `Boa noite, ${nombre}. Hora de ouvir sem pressa.`,
    (nombre: string) => `Boa noite, ${nombre}. Sua música pode desacelerar.`,
    (nombre: string) => `Feche o dia com a música certa, ${nombre}.`,
    (nombre: string) => `Que toque algo bem escolhido esta noite, ${nombre}.`,
    (nombre: string) => `Desacelere, ${nombre}, e deixe sua biblioteca tocar.`,
    (nombre: string) => `${nombre}, esta noite merece uma seleção caprichada.`,
    (nombre: string) => `Modo noturno, ${nombre}. Melhor com uma seleção tranquila.`,
    (nombre: string) => `Boa noite, ${nombre}. O melhor fim de dia começa com play.`,
    (nombre: string) => `${nombre}, deixe uma boa música fazer o resto.`,
    (nombre: string) => `É o seu momento, ${nombre}: calma, ritmo e boa música.`,
    (nombre: string) => `Que a noite encontre a música certa, ${nombre}.`,
    (nombre: string) => `${nombre}, coloque música neste trecho do dia.`,
    (nombre: string) => `Boa noite, ${nombre}. Há álbuns para ouvir com calma.`,
    (nombre: string) => `${nombre}, volte às suas músicas de confiança.`,
    (nombre: string) => `A noite pede uma escuta mais cuidadosa, ${nombre}.`,
    (nombre: string) => `Boa noite, ${nombre}. Sua biblioteca continua acordada.`
  ],
  sinNombre: [
    'O que você quer ouvir hoje?',
    'Dê play em algo que acompanhe este momento.',
    'Sua próxima música favorita pode começar agora.',
    'Hoje é um ótimo dia para descobrir música nova.',
    'Sua biblioteca está pronta: só falta você escolher.',
    'Faça este momento soar melhor.',
    'Explore algo novo: uma música pode surpreender você.',
    'Escolha um ritmo e deixe tudo se encaixar.',
    'Pronto para uma sessão que valha a pena?',
    'Aperte play: as boas descobertas começam aqui.',
    'A trilha sonora de hoje está esperando por você.',
    'Uma boa música sempre chega na hora.',
    'Dê uma chance àquela faixa que você ainda não ouviu.',
    'Experimente uma seleção diferente sem sair da sua biblioteca.',
    'Hoje também pode soar incrível.',
    'Abra a sessão: há música para cada momento.',
    'Volte ao que você estava ouvindo.',
    'Há favoritas prontas para tocar de novo.',
    'Sua biblioteca tem algo para esta hora.',
    'Escolha uma música e deixe a sessão seguir.',
    'Redescubra um álbum que você não ouve há tempos.',
    'Há artistas esperando outra volta.',
    'Sua música local tem muito a mostrar.'
  ]
};
