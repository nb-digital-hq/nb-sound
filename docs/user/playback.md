# Escuchar

> **Estado:** Implementado.

## La barra de reproducción

* **A la izquierda:** la portada, el título, el artista y el álbum. La portada y el título abren la reproducción
  ampliada; el artista y el álbum llevan a sus páginas.
* **En el centro:** aleatorio, anterior, reproducir o pausar, siguiente y repetir (la cola o una canción). Debajo, la
  forma de onda de la canción, que es también la barra de progreso.
* **A la derecha:** letras, karaoke, la cola con su número, el volumen, pantalla completa y el menú **···**: añadir a
  una playlist, favorita, minirreproductor y **dónde suena** (este equipo o el teléfono que lo controla).

Cuando la ventana es estrecha, lo que no cabe pasa a una fila arriba del menú ···; nada se pierde.

## La reproducción ampliada

Desde la portada de la barra o con F. Dos modos: **portada** (grande, sobre el color de la canción, con sus datos) y
**letras** (sincronizadas, con la línea que suena destacada). **Pantalla completa** con F11: si el ratón se queda
quieto 3,5 segundos, desaparecen los controles.

## La cola

Desde la barra o con Ctrl+Q: las canciones que vienen, aunque sean miles, con la que suena marcada; reproducir o
quitar cada una, reordenar arrastrando o con Alt+flechas, y vaciarla.

## El minirreproductor

Ctrl+M o el menú ···: una ventana pequeña con la portada, la canción, los controles y el progreso. Pide quedarse
encima de las demás (en Wayland lo decide el escritorio). Cerrarla solo la esconde: volver a abrirla es instantáneo.

## El sonido

* **Sin cortes entre canciones:** lo que suena es exactamente lo que hay en los archivos.
* **Igualar el volumen entre canciones** (EBU R128), en Ajustes, Reproducción.
* **Ecualizador** de diez bandas con 18 preselecciones, en Ajustes, Reproducción.
* Formatos: MP3, FLAC, M4A (AAC y ALAC), OGG Vorbis y WAV.

## Controles del sistema

Las teclas multimedia, los controles del escritorio (MPRIS en Linux, el panel de multimedia de Windows y el Centro de
control de macOS) y el icono de la bandeja manejan NB Sound sin abrir la ventana. Cerrar la ventana cierra la
aplicación.

## Lo que recuerda

Al volver a abrir: la canción, el segundo exacto (en pausa), la cola, el aleatorio, la repetición y el volumen. Una
escucha cuenta en tu historial desde los 30 segundos (o la mitad, en canciones de menos de un minuto).
