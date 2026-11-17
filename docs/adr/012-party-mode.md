# ADR-012: Modelo de confianza y aislamiento del Modo fiesta

**Fecha:** 2026-10-31
**Estado:** Aceptado

## Contexto

En el Modo fiesta, los invitados presentes en la misma red escanean un código QR desde el navegador de su teléfono,
buscan en el catálogo del anfitrión y sugieren qué suena después. A diferencia de NB Sound Mobile
([ADR-005](005-mobile-contract.md)), cuyo teléfono es del propio usuario y está emparejado con huella fijada, aquí los
clientes son **desconocidos en una red compartida**:

1. Los navegadores móviles muestran una advertencia severa ante un certificado autofirmado en una IP privada. Para
   que cualquiera entre escaneando un código, sin instalar nada, la página se sirve por HTTP en un puerto propio.
2. Cualquier dispositivo de la misma subred puede alcanzar el puerto mientras dura la fiesta.
3. Riesgos: acceso a la sincronización, al audio o al control del reproductor; inundación de sugerencias o búsquedas
   costosas; suplantación entre invitados; exposición de rutas, historial o claves; un servidor que queda abierto.

## Alternativas consideradas

1. **Un rol de invitado en el servidor móvil.** Rechazada: obliga a los navegadores a atravesar la advertencia del
   certificado y mezcla en el mismo enrutador las rutas de control total con las abiertas a desconocidos.
2. **Encolar todo lo que pida cualquiera.** Rechazada: un solo invitado secuestra la sesión.
3. **Un servidor HTTP efímero y aislado, con un token de sesión, rutas mínimas, cuotas y aprobación del
   anfitrión.** Aceptada.

## Decisión

1. **Proceso de red separado**: un servidor `axum` propio en un puerto alto distinto del rango del servidor móvil, que
   **solo escucha mientras el Modo fiesta está encendido**. Apagarlo o cerrar la aplicación cierra el puerto y borra
   todo su estado en memoria.
2. **Token de sesión**: 128 bits del generador del sistema en la URL del QR, comparados en tiempo constante. La página
   es un único documento embebido en el binario, sin peticiones externas, y emite una cookie `HttpOnly;
   SameSite=Strict` con un identificador de invitado. Rotar el QR invalida el token y las cookies anteriores.
3. **Superficie mínima, sin audio**: la página, el estado (qué suena y las sugerencias), la búsqueda (como mucho 25
   resultados, sin rutas), sugerir (cuerpo de 512 bytes como máximo) y la miniatura de 64 px de una pista de la
   biblioteca. No existe ninguna ruta que sirva audio ni portadas originales.
4. **Cuotas en memoria**: 3 sugerencias pendientes por invitado (cookie e IP), 10 segundos entre envíos, 50 pendientes
   en total y 30 peticiones por minuto por IP. Ninguna sugerencia escribe en la base.
5. **El anfitrión decide**: de fábrica, cada sugerencia espera a que la acepte, la descarte o bloquee al invitado. Con
   "Encolar sugerencias automáticamente" entran solas, siempre al final de la cola: nunca interrumpen la canción que
   suena ni cambian el volumen.
6. **La página habla el idioma del navegador del invitado** ([ADR-011](011-internationalization.md)).

## Consecuencias

- **Positivas:** cualquier teléfono entra sin instalar nada ni aceptar advertencias; un invitado nunca descarga
  música, lee el historial ni controla el reproductor; la base no recibe ninguna escritura desde la red.
- **Negativas:** el tráfico va por HTTP dentro de la red local; por él solo viajan títulos de la búsqueda y el
  identificador de la pista sugerida.
- **Riesgos y cómo se vigilan:** que se comparta la URL o se borren las cookies para esquivar la cuota; la cuota es
  también por IP, el modo moderado es el de fábrica y el QR se rota con un clic.
