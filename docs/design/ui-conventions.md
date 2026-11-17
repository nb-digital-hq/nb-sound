# Convenciones de interfaz

> **Estado:** Aplicable. Las vigila `scripts/check-ui.mjs` (en `pnpm guardianes` y en el gancho de pre-commit).

| Regla | Por qué |
|---|---|
| Ningún color literal: solo tokens `--m3c-*` | Los 63 temas y el color por portada tienen que poder cambiar todo |
| Sin desenfoques (`backdrop-filter`) en el cromo permanente | En WebKitGTK cuestan fotogramas en cada desplazamiento |
| Sin `will-change` | Reserva memoria de composición para siempre |
| Nada se anima en reposo | Una animación permanente gasta CPU y batería sin que nadie la mire |
| Ningún `:hover` mueve la caja del elemento | Mover lo que está bajo el ratón provoca parpadeos y clics fallidos |
| Iconos SVG del catálogo, nunca emojis ni letras como iconos | Coherencia y accesibilidad |
| Ninguna acción exclusiva del ratón | Todo se puede hacer con teclado: las filas con Enter, los menús con flechas y Escape |
| El foco siempre visible | Navegación con teclado |
| Ninguna clase pintada sin regla de CSS | Una clase sin regla es un defecto que no se ve |
| Ningún medio servido desde el esquema propio de la aplicación | WebKitGTK no lo reproduce; el audio es del motor de Rust |
| Ningún texto en el marcado | Todo sale de los diccionarios ([i18n](../architecture/i18n.md)); `check-i18n` |
| Los componentes del sistema de diseño se usan | Uno sin usar es mantenimiento sin función |

## Disposición

* La ventana tiene cuatro partes fijas (barra lateral, barra superior, barra de reproducción y el lienzo) y solo el
  lienzo se desplaza. Las barras son opacas.
* La barra lateral se contrae a iconos en ventanas estrechas o a petición, y lo recuerda.
* La barra de reproducción decide qué cabe por su propio ancho: por debajo de 832 px, letras y pantalla completa
  pasan al menú ···; por debajo de 704 px, también aleatorio, repetir y karaoke. Nada se pierde ni se pisa.
* Las consolas (karaoke, dispositivos, añadir música) entran desde la derecha y son modales.

## Textos

* Escritos para personas: sin códigos, sin nombres de bibliotecas, sin causas internas.
* Lo que falla se dice con lo que se puede hacer ("Activa los servicios en línea en Privacidad para descargarlo").
* Lo que se hace en lote se confirma antes; lo de una fila se hace y se dice después.
