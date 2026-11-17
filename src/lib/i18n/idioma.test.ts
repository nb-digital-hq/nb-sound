import { describe, expect, it } from 'vitest';
import { textosAparte } from './aparte';
import { esIdioma, idioma, idiomaDelSistema, localeActual, textos, usarIdioma } from './index';

describe('el idioma de la interfaz', () => {
  it('un grupo tomado antes de cambiar de idioma habla el idioma nuevo', async () => {
    const nav = textos.nav;
    await usarIdioma('es');
    expect(nav.ajustes).toBe('Ajustes');
    await usarIdioma('en');
    expect(nav.ajustes).toBe('Settings');
    await usarIdioma('pt');
    expect(nav.ajustes).toBe('Configurações');
    expect(localeActual()).toBe('pt-BR');
    await usarIdioma('es');
  });

  it('los diccionarios diferidos siguen al mismo idioma', async () => {
    await usarIdioma('en');
    expect(textosAparte.copia.titulo).toBe('Backup');
    await usarIdioma('es');
    expect(textosAparte.copia.titulo).toBe('Copia de seguridad');
  });

  it('las funciones de texto llegan en el idioma en uso', async () => {
    await usarIdioma('en');
    expect(textos.biblioteca.cuentaPistas(2)).toBe('2 tracks');
    await usarIdioma('es');
    expect(textos.biblioteca.cuentaPistas(2)).toBe('2 pistas');
  });

  it('un idioma que no se habla cae en el base sin romper la interfaz', async () => {
    expect(esIdioma('fr')).toBe(false);
    await usarIdioma('fr');
    expect(idioma()).toBe('es');
    expect(textos.nav.inicio).toBe('Inicio');
  });

  it('los grupos se pueden recorrer como objetos', async () => {
    await usarIdioma('en');
    expect(Object.keys(textos.porRevisar.grupos)).toContain('danadas');
    expect(Object.entries(textos.biblioteca.tipos)).toContainEqual(['Single', 'Single']);
    await usarIdioma('es');
  });

  it('el idioma del sistema se reconoce o cae en inglés', () => {
    expect(['es', 'en', 'pt']).toContain(idiomaDelSistema());
  });
});

describe('los rótulos fijos de la navegación', () => {
  it('siguen al idioma después de cargarse el módulo', async () => {
    const { DESTINOS, SECCIONES } = await import('#lib/core/navegacion.ts');
    await usarIdioma('en');
    expect(DESTINOS[0]?.etiqueta).toBe('Home');
    expect(SECCIONES[0]?.etiqueta).toBe('Explore');
    await usarIdioma('es');
    expect(DESTINOS[0]?.etiqueta).toBe('Inicio');
  });
});
