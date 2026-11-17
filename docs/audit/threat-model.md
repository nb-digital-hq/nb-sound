# Modelo de amenazas

> **Estado:** Implementado. Método: STRIDE sobre cada superficie expuesta.

## Activos

| Activo | Por qué importa |
|---|---|
| La biblioteca y la carpeta de la música | Es de la persona; perderla o alterarla es el peor fallo posible |
| El historial y el perfil | Dicen mucho de quien escucha |
| Las claves de servicios | Dan acceso a cuentas de terceros (IA) |
| La identidad del servidor móvil y los `device_token` | Dan acceso a la biblioteca desde la red local |
| Las autorizaciones de NB Launcher | Dan acceso a la frecuencia de las copias |
| Las claves de firma de las publicaciones | Permitirían entregar código como si fuera oficial |

## Superficies y defensas

| Superficie | Amenaza | Defensa |
|---|---|---|
| Archivos importados | Un archivo malformado que rompa el decodificador o el lector de etiquetas | Decodificadores en Rust seguro (Symphonia, `lofty`); un fallo deja la pista en Por revisar sin tirar la aplicación |
| Rutas desde la interfaz | Recorrido de directorios, enlaces simbólicos que escapan | Los selectores se abren desde Rust; toda ruta se canonicaliza y se acota (`security::resolve_within`, `check-rutas`) |
| Servidor móvil (red local) | Acceso sin emparejar, suplantación, lectura del tráfico | TLS con certificado propio fijado por huella; token de emparejamiento de un solo uso y 300 s; `device_token` comparado por huella; quitar un teléfono cierra su sesión |
| Modo fiesta (red local, HTTP) | Acceso a audio o control, inundación, suplantación | Servidor aparte que solo escucha encendido; token de 128 bits; sin rutas de audio; cuotas por invitado e IP; el anfitrión aprueba ([ADR-012](../adr/012-party-mode.md)) |
| Enlace de NB Launcher | Un proceso local que se haga pasar por NB Launcher | Desafío Ed25519 de un solo uso ligado al PID; consentimiento en NB Sound; ámbito limitado a la frecuencia; revocación persistente ([ADR-017](../adr/017-launcher-persistent-authorization.md)) |
| Salidas a internet | Fuga de datos, servicios no consentidos | Puerta única con consentimiento por servicio y anfitriones permitidos; sin telemetría ([red](../architecture/network.md)) |
| Respuestas de servicios | Datos maliciosos o enormes | Tope de tamaño en memoria; JSON tipado; las imágenes se decodifican con `image` |
| Agregados descargados | Un archivo alterado | URL, tamaño y SHA-256 compilados en el binario; se verifica antes de instalar |
| Actualizaciones | Un paquete alterado o de otro origen | Firma minisign contra la clave pública compilada; el `.tar.gz` se verifica en memoria y nunca pasa por `/tmp` |
| Página del Modo fiesta | Inyección en el navegador del invitado | Documento estático embebido, `Content-Security-Policy`, los textos se insertan con `textContent` |
| Webview | Código de terceros en la interfaz | Sin contenido remoto; CSP de Tauri; el webview no accede al disco ni a la red |

## Lo que no se defiende

* Un atacante con la misma cuenta de usuario o con permisos de administrador en el equipo: puede leer los datos de la
  aplicación como la propia persona.
* La red local donde está el Modo fiesta, mientras está encendido: por HTTP solo viajan títulos de la búsqueda y el
  identificador de la canción sugerida.
* Las aplicaciones de terceros a las que se conecta (los servicios en línea), más allá de no enviarles nada sin
  permiso.
