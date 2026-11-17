/** Textos en portugués de Brasil del consentimiento y los permisos de NB Launcher. */

import type { TextosLauncher } from './es-launcher';
import type { Forma } from './forma';

export const ptLauncher: Forma<TextosLauncher> = {
  titulo: 'Acesso do NB Launcher',
  descripcion: 'Autorizações lembradas para a frequência dos seus backups.',
  solicitud: 'Permitir acesso pelo NB Launcher',
  alcance: (cambiar: boolean, observar: boolean) =>
    `A solicitação local permite consultar a frequência${cambiar ? ', alterá-la' : ''}${observar ? ' e receber suas mudanças' : ''}. O NB Sound vai lembrar esta permissão neste computador. Você pode revogá-la em Configurações → Privacidade.`,
  permitir: 'Permitir e lembrar',
  recordadas: (n: number) => (n === 1 ? '1 autorização lembrada' : `${n} autorizações lembradas`),
  ninguna: 'Não há autorizações lembradas.',
  revocadas: 'Há autorizações revogadas. Essas identidades não podem se conectar de novo.',
  revocar: 'Revogar autorizações lembradas',
  efectoRevocar: 'As sessões abertas com essas autorizações serão fechadas.',
  permitirSolicitudes: 'Permitir solicitações de novo',
  efectoPermitir: 'Vai pedir sua confirmação de novo; isso não concede acesso.',
  noDisponible: 'Não é possível solicitar novas autorizações nesta sessão.',
  fallo: 'Não foi possível confirmar a permissão do NB Launcher.',
  trabajando: 'Salvando…'
};
