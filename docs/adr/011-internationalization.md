# ADR-011: Tres idiomas, con cambio al momento

**Fecha:** 2026-11-16
**Estado:** Aceptado

## Contexto

La interfaz sale entera de diccionarios (`check-i18n` rechaza texto incrustado en el marcado), pero solo había uno,
en español. Para publicar fuera hacen falta al menos inglés y español, y la suite habla también portugués. Cambiar
de idioma no debe obligar a reiniciar, igual que ningún otro ajuste de NB Sound. Unas pocas cosas las nombra el
runtime y no la interfaz: la bandeja, los diálogos nativos, las playlists que crea NB Sound, la página del Modo
fiesta y la tarjeta del resumen de escucha.

## Alternativas consideradas

1. **Una biblioteca de internacionalización** (formato ICU o similar). Rechazada: dependencia nueva para lo que ya
   resuelven los diccionarios tipados, y perdería la comprobación de claves en compilación.
2. **Leer los textos como función** (`textos()`) en cada sitio. Rechazada: más de doscientos sitios que cambiar y una
   regla fácil de olvidar en un componente nuevo.
3. **Diccionarios tipados y una cara reactiva que resuelve al leer.** Aceptada.

## Decisión

* **Español, inglés y portugués de Brasil.** El español es la fuente: la forma de los demás sale de él
  (`Forma<Textos>`), así que una clave que falte en otro idioma no compila.
* **Cambio al momento.** Cada diccionario se registra y se lee a través de un proxy que resuelve contra el idioma en
  uso, que es un `$state` de Svelte: toda lectura en una plantilla queda suscrita y la interfaz se repinta sola. El
  cambio espera a tener todos los diccionarios del nuevo idioma para no enseñar una pantalla mezclada.
* **Carga a demanda.** El español viaja en el arranque; el inglés y el portugués se cargan al elegirlos, para no
  pagar en cada apertura dos idiomas que la persona no usa (`check-paquete`).
* **Sin destello al abrir.** El idioma se guarda en la caché local del webview y `+layout.ts` lo aplica antes de
  pintar; la preferencia `idioma` de la base manda cuando responde, porque es la que sobrevive a una copia de
  seguridad. La primera vez se usa el idioma del sistema si es uno de los tres, y si no, el inglés.
* **El runtime habla el mismo idioma.** La preferencia `idioma` fija el del proceso (`idioma.rs`); al cambiarla se
  vuelve a escribir el menú de la bandeja y se renombran las playlists que nombra NB Sound, salvo las que la persona
  editó. El teléfono recibe los nombres nuevos en la siguiente sincronización. La página del Modo fiesta habla el
  idioma del navegador de cada invitado, porque es su teléfono.
* **Las palabras del DJ y de Buscar se entienden en los tres idiomas.** Las fichas, los ejemplos y las ideas de cada
  diccionario los comprueba una prueba contra el motor que los interpreta.

## Consecuencias

- **Positivas:** tres idiomas sin dependencias nuevas; cambiar de idioma es un clic.
- **Negativas:** cada texto nuevo se escribe tres veces.
- **Riesgos y cómo se vigilan:** que un texto guardado en una variable al montarse no cambie; los grupos son proxies
  que resuelven al leer, y las pruebas de `idioma.test.ts` lo comprueban.
