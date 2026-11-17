# ADR-018: DJ Mashups, un compositor automático fuera de línea

**Fecha:** 2026-11-14
**Estado:** Aceptado

## Contexto

El DJ Privado encadena canciones completas con dos platos en tiempo real. Un mashup combina fragmentos y capas de
muchas canciones (la voz de una sobre el instrumental de otra), y eso exige tres cosas: una rejilla temporal por
pista (pulsos, compases, secciones), los cuatro stems de HTDemucs (el karaoke solo usaba la voz) y cambiar tempo y
tono sin tocar la otra magnitud. El hilo de audio del motor ([ADR-003](003-audio-engine.md)) no es el sitio: un
mashup denso tiene decenas de capas y transformaciones caras.

## Alternativas consideradas

1. **Componer en tiempo real sobre los platos del DJ.** Rechazada: dos platos no bastan y el estiramiento no cabe en
   el hilo de audio sin riesgo de cortes.
2. **Un modelo generativo.** Rechazada: ninguno local con licencia y tamaño compatibles, y no sería reproducible.
3. **Rubber Band o signalsmith-stretch.** Rechazada: C++ en la CI de tres plataformas; el compositor limita las
   transformaciones a un rango donde una implementación propia basta.
4. **Stems en WAV de 32 bits.** Rechazada: unos 40 MB por canción; FLAC de 16 bits ocupa cuatro veces menos sin
   pérdida audible para remezclar.
5. **Un segundo recorrido de Demucs.** Rechazada: se generaliza el del karaoke.

## Decisión

1. **Una pestaña.** DJ Mashups vive en el destino DJ junto al DJ Privado; un botón en la cabecera alterna la
   experiencia. El historial es uno solo y lista sesiones y mashups con su tipo.
2. **Separación generalizada.** `karaoke::separar_stems` devuelve los stems pedidos y el karaoke es ese recorrido
   pidiendo la voz. Nunca hay dos sesiones de HTDemucs en memoria: comparten un turno.
3. **Una caché de stems.** FLAC de 16 bits en la caché de la aplicación, por huella de contenido, modelo y versión del
   separador, con escritura atómica y un tope configurable (5 GB de fábrica) que se poda por antigüedad de uso. El
   karaoke saca su instrumental de la misma caché, así que cada canción se separa una vez para los dos. Un mashup
   separa solo los tramos que su plan previo va a usar. Dependencia: `flacenc` (Apache-2.0, Rust puro).
4. **Análisis estructural** sobre la pista entera: pulsos afinados, tempo, compases, secciones por novedad sobre croma
   y energía, y actividad vocal; cada nivel con su confianza. Sin pulso fiable no se inventa una rejilla.
5. **Estiramiento propio**: vocoder de fase con bloqueo de fase y reinicio en los transitorios, sobre `rustfft`; el
   tono es estirar y remuestrear con `rubato`, con corrección de la envolvente cepstral en voces y mezclas para
   conservar los formantes. Se transpone antes el acompañamiento que la voz. Límites: ±8 % de tempo y ±2 semitonos.
6. **Compositor determinista**: búsqueda por haces sobre frases de 4, 8 o 16 compases con una puntuación explícita
   (curva de energía, armonía, coste de transformación, variedad, cobertura, continuidad, densidad). La
   compatibilidad decide cómo entra cada canción, no si entra: superponer, pasar de una base a otra y reproducir la
   mezcla original son compatibilidades distintas, y las transiciones (intercambio de graves con anticipo, corte en
   el primer tiempo en una frontera estructural, cruce) se eligen por la armonía en la frontera, la estructura y la
   fase rítmica de dos compases. Solo el tempo, la transposición, dos voces a la vez y un choque armónico o de clave
   con alta confianza son restricciones estrictas. La semilla solo desempata; con la misma semilla y los mismos
   datos, el plan y el audio son idénticos. Sin candidatos aceptables, cede y lo dice; si la duración no admite las
   canciones pedidas de forma reconocible, lo dice con lo que haría falta. Sin aprendizaje automático: sin un
   conjunto perceptual propio no se puede comprobar que mida compatibilidad entre canciones.
7. **Render fuera de línea** en un hilo de fondo: capa a capa, igualación a −18 LUFS por capa, capas partidas en
   graves y resto para las transiciones, atenuación de la banda de presencia del acompañamiento solo donde tapa a la
   voz, dos pasadas para llevar el total a −14 LUFS, un limitador de pico con anticipo, el limitador del motor y un
   WAV de 16 bits que incluye la cola del cierre. Exportar copia ese WAV adonde diga el
   selector del sistema, nunca dentro de la carpeta de la música.
8. **Persistencia** (V13): la petición, la semilla, el plan entero, el archivo y las métricas de cada mashup, y sus
   canciones con título y artista, que sobreviven a borrar la pista.
9. **Escucha** por el reproductor de siempre: la cola espera en pausa y un mashup no cuenta como escucha de ninguna
   canción.
10. **Límites declarados.** Hasta unas 30 canciones con capas; de 50 a 100 con frases cortas (estilo Relámpago) y
    menos superposición. La separación domina el tiempo de la primera vez.

## Consecuencias

- **Positivas:** sin red ni instalaciones; reproducible; reutiliza el motor, el karaoke y el DJ.
- **Negativas:** la primera vez tarda lo que tarda separar; la caché lo convierte en segundos.
- **Riesgos y cómo se vigilan:** artefactos del estiramiento en voces reales al límite del rango; los límites del
  compositor lo acotan y la [arquitectura de los mashups](../architecture/mashups.md) recoge lo medido.
