/** Textos en inglés del consentimiento y los permisos de NB Launcher. */

import type { TextosLauncher } from './es-launcher';
import type { Forma } from './forma';

export const enLauncher: Forma<TextosLauncher> = {
  titulo: 'NB Launcher access',
  descripcion: 'Remembered authorizations for the frequency of your backups.',
  solicitud: 'Allow access from NB Launcher',
  alcance: (cambiar: boolean, observar: boolean) =>
    `The local request lets it read the frequency${cambiar ? ', change it' : ''}${observar ? ' and receive its changes' : ''}. NB Sound will remember this permission on this computer. You can revoke it in Settings → Privacy.`,
  permitir: 'Allow and remember',
  recordadas: (n: number) =>
    n === 1 ? '1 remembered authorization' : `${n} remembered authorizations`,
  ninguna: 'There are no remembered authorizations.',
  revocadas: 'Some authorizations were revoked. Those identities cannot connect again.',
  revocar: 'Revoke remembered authorizations',
  efectoRevocar: 'Sessions opened with these authorizations will be closed.',
  permitirSolicitudes: 'Allow requests again',
  efectoPermitir: 'It will ask for your confirmation again; this does not grant access.',
  noDisponible: 'New authorizations cannot be requested in this session.',
  fallo: 'The NB Launcher permission could not be confirmed.',
  trabajando: 'Saving…'
};
