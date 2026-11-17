# Cambios de NB Sound

## 1.0.0 — 2026-11-17

Primera versión de NB Sound.

* Reproductor con motor de audio propio: sin cortes entre canciones, sonoridad igualada con EBU R128, ecualizador de
  diez bandas con 18 preselecciones, forma de onda como barra de progreso, reproducción ampliada con letras
  sincronizadas, pantalla completa, cola de miles de canciones, minirreproductor y controles del sistema (MPRIS,
  panel multimedia de Windows y Centro de control de macOS).
* Biblioteca de MP3, FLAC, M4A, OGG Vorbis y WAV: importa, identifica (AcoustID, MusicBrainz, iTunes),
  etiqueta, organiza en la carpeta de la música sin convertir nunca de formato, completa portadas, letras e imágenes
  de artista, aparta duplicados y vigila la carpeta. Lo dudoso queda en Por revisar, agrupado por motivo.
* Playlists manuales, inteligentes por regla y creadas para ti, que siguen al idioma de la interfaz.
* Tu música: identidad musical, actividad, tops y resumen local de escucha exportable como tarjeta.
* DJ Privado: una sesión mezclada a partir de una frase en español, inglés o portugués, con cinco técnicas de
  transición.
* DJ Mashups: un compositor automático que crea piezas nuevas con fragmentos de tus canciones, al mismo tempo y en
  tonalidades que encajan. Elige qué canciones y cómo entra cada una (voz sobre otra base, mezcla original,
  intercambio de graves o corte en un cambio de sección), respeta las pausas de las voces y la clave de dos compases,
  conserva los formantes al transponer, abre y cierra con intención, y dice cuándo la duración no da para tantas
  canciones. Reproducible por semilla y exportable a WAV.
* Karaoke de cualquier canción, separando la voz en el equipo; instrumentales en FLAC compartidos con los mashups.
* A ciegas, un juego para redescubrir la biblioteca por la portada o por el sonido.
* NB Sound Mobile: sincronización de biblioteca, playlists, letras e instrumentales por la red local y control remoto,
  con el protocolo `1`.
* Copia a una tarjeta o un USB y Modo fiesta para que los invitados sugieran canciones desde su navegador.
* Agregados que se instalan con un clic: karaoke, análisis musical, pedidos al DJ en tus palabras y aceleración por
  GPU (Linux con NVIDIA).
* Interfaz en español, inglés y portugués, con cambio al momento; 63 temas, cinco tipografías y seis tamaños.
* Integración con NB Launcher: frecuencia de copias compartida, autorización persistente, instalación administrada y
  catálogo firmado.
* Una sola puerta de red con interruptor general y consentimiento por servicio; secretos en el llavero del sistema;
  sin telemetría.
* Copia de seguridad en un archivo, con copias automáticas.
* Paquetes para Linux (`.deb`, `.rpm`, `.tar.gz` y AppImage), verificados en un equipo real. Paquetes **sin probar en
  un equipo real** para Windows (`-setup.exe` y `.msi`) y macOS en Apple Silicon (`.dmg`); no llevan firma de
  Microsoft ni de Apple, así que Windows muestra el aviso de SmartScreen y macOS pide autorizar la aplicación al
  abrirla. Todos firmados para el actualizador y en una lista de huellas `SHA256SUMS` firmada con Ed25519.
