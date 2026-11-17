# DJ Mashups

> **Estado:** Implementado. Decisión: [ADR-018](../adr/018-dj-mashups.md). Lo que ve la persona: [DJ](../user/dj.md).

## 1. Recorrido de una generación

| Etapa | Módulo | Qué hace | Qué reutiliza |
|---|---|---|---|
| Elegir | `mashup/seleccion.rs` | Prefiltro por tempo con los rasgos guardados alrededor de un ancla; cupo de `1,5 × canciones + 4` (máx. 160), ampliado una vez al doble si el plan previo no llega a las canciones pedidas | `dj::planificador::cargar`, Camelot de `dj::transiciones`, `dj::azar` |
| Analizar | `mashup/estructura.rs` | Pulsos, compases, secciones, energía, croma y patrón de ataques por compás; caché en `estructura_pistas` | Espectrograma, mel, envolvente, `tempo` y `pulsos` de `rasgos.rs`; `rasgos::cargar_mono` |
| Plan previo | `mashup/compositor.rs` | Un plan que supone stems para todas dice qué pistas va a usar el mashup y cuáles suenan solo con su mezcla original | — |
| Separar | `karaoke::separar_stems` + `mashup/stems.rs` | Cuatro stems solo de las pistas que el plan previo usa como stems; FLAC en caché; actividad vocal por cuarto de compás | El único recorrido de HTDemucs, su sesión ONNX y su turno |
| Componer | `mashup/compositor.rs` | Búsqueda por haces sobre frases, con stems | — |
| Mezclar | `mashup/render.rs` + `mashup/estiramiento.rs` | Capas, tempo y tono, fundidos, R128, limitador, WAV | `audio::limitador::limitar`, `audio::sonoridad::OBJETIVO_LUFS`, `ebur128`, `rubato`, `Decodificador` |
| Guardar | `mashup/persistencia.rs` | Petición, semilla, plan, métricas y pistas | `db::transaccion_de_escritura` |

`mashup/servicio.rs` orquesta las etapas en un hilo `mashup` de prioridad baja (`prioridad.rs`); una sola generación
a la vez. El progreso se publica por `mashup:progreso` como mucho una vez por punto porcentual o al cambiar de etapa
o de pista, y el final por `mashup:terminado`.

## 2. Análisis estructural

* **Señal.** La pista entera en mono a 22.050 Hz (los rasgos usan los primeros 90 s). El espectrograma se reduce
  marco a marco a ataques, graves, RMS y croma y se suelta: de unos 85 MB por pista de 4 min a menos de 2 MB.
* **Tempo.** El de `rasgos` (autocorrelación con prior en 120 BPM), plegado a [78, 156). Como sale de un retardo
  entero (a 120 BPM los vecinos son 117,5 y 123,0), se afina con la pendiente de una regresión sobre los pulsos;
  solo se acepta si confirma el grueso (±4 %).
* **Pulsos.** El seguidor de `rasgos` con el tempo plegado; cada pulso se afina con interpolación parabólica sobre la
  envolvente y se corrige el retardo de un marco de la envolvente (medido: 25 ms sin corregir, < 7 ms corregido).
* **Confianza del pulso.** La autocorrelación normalizada de la envolvente al periodo del pulso. La regularidad de los
  pulsos no sirve: el seguidor impone su periodo aunque no haya pulso. Clics: > 0,6; ruido con un tono: < 0,15.
  Por debajo de 0,35 la rejilla no es fiable, el compositor no usa la pista y sus compases son de 2 s con confianza 0.
* **Compás.** Se asume 4/4. La fase de cuatro con más subida de graves y más cambio de croma es el primer tiempo.
* **Secciones.** Novedad de Foote con un núcleo de 4 + 4 compases sobre una autosimilitud de croma (70 %) y energía
  (30 %); fronteras en los máximos por encima de media + 0,5 σ, atraídas a la frase de 4 compases, a 4 compases como
  mínimo. Se agrupan por parecido (coseno > 0,92 y energía a menos de 0,2); el coro es el grupo repetido más fuerte,
  la intro y el final son los extremos flojos, lo repetido es verso y lo único, puente. Cada sección lleva confianza.
* **Patrón rítmico.** Por compás, la envolvente de ataques muestreada en las 16 semicorcheas de la rejilla (máximo a
  ±1 marco), normalizada con el percentil 95 de la pista y guardada en un byte por paso: 16 bytes por compás. Es la
  representación de patrones de inicio por subdivisión métrica habitual en los trabajos de groove y de clave; aquí no
  se clasifica: el compositor compara patrones entre pistas (sección 4).
* **Voz.** Con el stem de voz, marco a marco (23 ms) y resumida por cuarto de compás y por compás como fracción de
  tiempo cantado. Umbral adaptativo a la propia voz (piso de ruido, percentil 20, más 12 dB, sin bajar de 24 dB bajo
  el percentil 95), histéresis de 6 dB para que una vocal sostenida que baja no se parta, pausas de menos de 250 ms
  (respiraciones, consonantes sordas) que no terminan la frase y 150 ms de cola tras la última sílaba. La RMS por
  compás que se usaba antes no distinguía un compás cantado con una respiración de uno mudo con una sílaba final;
  lo que importa al cortar es si se canta justo en la frontera.
* **Versión.** La estructura es la versión 2: una fila anterior caduca y se recalcula (0,5 s de CPU por minuto de
  audio); los stems de la caché no se tocan y la actividad vocal se vuelve a medir desde su FLAC.

## 3. Tempo y tono

Vocoder de fase propio (`estiramiento.rs`): ventana de Hann de 2.048 (46 ms), salto de síntesis de 512, bloqueo de
fase de identidad (Laroche y Dolson) y reinicio de fase en los transitorios. Un transitorio es un marco cuyo flujo
espectral **relativo** dobla la mediana local, es máximo local y trae al menos un 20 % de energía nueva: con el flujo
absoluto, las fluctuaciones de un tono sostenido pasaban por golpes y cada reinicio desafinaba (−150 cents a 110 Hz).
Los transitorios se deciden sobre la suma de canales y se aplican a los dos, para no mover la imagen estéreo. El tono
es estirar por `2^(s/12)` y remuestrear con sinc de `rubato` en la misma pasada. La identidad no toca la señal.

**Formantes.** Transponer estirando y remuestreando mueve también la envolvente espectral: una voz subida dos
semitonos suena un 12 % "más pequeña" (el efecto de ardilla), aunque su fundamental sea correcta. En voces y mezclas
transpuestas cada marco se corrige antes de sintetizar: envolvente por cepstro (logaritmo del módulo, transformada
inversa y lifter de 60 coeficientes, 1,4 ms) y módulo multiplicado por `E(r·f) / E(f)`, acotado a ±12 dB, para que
tras remuestrear los armónicos se muevan y la envolvente se quede. Es la corrección por envolvente cepstral descrita
por Röbel y Rodet (2005) en su forma simple, sin la iteración de la "true envelope". Con una vocal sintética a
+2 semitonos el centroide de formantes se desplaza un 7 % o más sin corrección y menos de un 5 % con ella; la
fundamental sube lo pedido (±20 cents). Cuesta dos FFT más por marco y solo en esas capas. Un instrumental no la usa.

**Primero se mueve el acompañamiento.** Para superponer una voz, el compositor elige la pareja de transposiciones
que cuesta menos con la voz tres veces más cara que la base, y entre las admisibles por Camelot decide el croma real
de los dos fragmentos (la tonalidad estimada puede fallar). En las mezclas reales medidas, de 45 voces superpuestas
solo 4 se transpusieron.

Límites: ±8 % de tempo, ±2 semitonos. La tonalidad nunca excluye una pista: decide cuánto se transpone y cuánto se
cobra, y si una voz puede ir encima.

## 4. El compositor

**Qué optimiza.** Se pide cuántas canciones, no cuáles: el compositor elige, entre las candidatas, las que mejor se
integran y en qué orden. La compatibilidad no es un requisito de entrada; decide **cómo** entra cada canción.

* **Tres compatibilidades distintas.** *Superponer* (voz de una sobre la base de otra) exige armonía compás a compás,
  pulsos fiables en las dos y que no crucen la clave. *Pasar* de una base a otra solo exige un buen punto de cambio.
  *Reproducir el original* solo exige el tempo. Una pareja que no se superpone es perfectamente válida en secuencia.
* **Estrictas y graduadas.** Son estrictas las restricciones que evitan defectos graves: ±8 % de tempo, ±2
  semitonos, nunca dos voces a la vez ni una voz sobre una mezcla con voz, nada de voz sobre una base cuyo croma
  medio baje de 0,6, cuya media compás a compás baje de 0,45 o con más del 30 % de compases que chocan (coseno
  < 0,5), con pulso < 0,5 o con conflicto de clave > 0,5. Todo lo demás se cobra en la puntuación.
* **Frases.** 4, 8 o 16 compases según el estilo; se acortan si no caben las canciones pedidas.
* **Formas de integrar una canción.** Su instrumental bajo la voz de otra; su voz sobre el instrumental de otra; su
  **mezcla original** sin nada encima (con o sin stems; reproducir el original es una opción, no un recurso de
  emergencia); su instrumental solo, como respiro; en Fiesta, su batería bajo la base de otra, solo si los grooves
  se parecen (correlación ≥ 0,5) y no cruzan la clave. Cada estilo tolera una fracción de frases con el original
  antes de cobrarlo (Equilibrado 30 %, Fiesta 20 %, Suave 45 %, Relámpago 35 %), y una racha de originales se cobra
  más cuanto más larga, para que el mashup no se vuelva una cadena de canciones cortadas.
* **Transiciones de base** (adaptación propia de técnicas de DJ, elegida por contexto):
  * *Intercambio de graves con anticipo* si la armonía de los dos últimos compases de la saliente y los dos primeros
    de la entrante casa (coseno ≥ 0,7), sin conflicto de clave y con compases previos disponibles (sin canto si
    suena su mezcla, no oídos ya): la entrante suena sin graves durante 2 compases (1 en Relámpago, 4 en Suave) y en
    el primer tiempo recibe los graves que la saliente suelta en ese instante; la saliente se desvanece en 1 compás.
  * *Corte en el primer tiempo* donde no casan: vale más cuanto más estructural es la frontera (la saliente acaba
    sección, la entrante la empieza, cambio claro de energía). Es lo que permite integrar parejas que no se
    superponen sin forzarlas.
  * *Cruce* corto de igual potencia en el resto, cobrado por la armonía que choca.
  * Seguir la misma grabación en su orden no es una transición: la capa se alarga.
* **Clave y fase de dos compases.** Para cada fragmento, los patrones medios de sus compases pares e impares,
  centrados y normalizados. Su correlación dice cuánto se organiza el ritmo en dos compases distintos (una clave
  marcada; nada en un bombo a negras). Entre dos fragmentos alineados se comparan las dos alineaciones: si la cruzada
  explica mejor que la directa, están en fases opuestas (una 3-2 contra una 2-3). El conflicto se pondera por la
  organización de dos compases de ambos y por la confianza de pulso y compás; si los dos tienen organización pero
  sus patrones no se parecen en ninguna fase, el resultado es *incierto*. Con conflicto alto no se superpone una voz
  ni una batería; con duda se cobra y se prefieren cortes a solapes. No hay un clasificador de clave: solo una
  comparación relativa con su confianza. Pop, rock y electrónica, que repiten cada compás, no se ven afectados.
* **Voces y cortes.** De la actividad por cuarto de compás sale, para cada frontera, si una frase cantada la cruza.
  Cambiar de voz con la saliente a media frase se cobra 0,9 y entrar a media frase 0,4, así que los relevos caen en
  las pausas (en la prueba sintética, al menos el 80 %). Soltar una voz a media frase sin relevo es barato si la
  frase siguiente no trae voz propia: la capa sigue sonando con su cola hasta que la voz calla (hasta 2 compases).
  Entrar en una mezcla original cortando su voz se cobra.
* **Apertura y cierre.** La primera frase prefiere el arranque real de una canción o una intro (entra tal cual);
  si no lo hay, entra filtrada un compás, sin graves. La última prefiere un final real o un final de sección y
  cobra cortar una voz; si no hay final real, la base se desvanece dos compases (uno en Relámpago) **después** del
  último compás y la voz termina con su cola. Antes, la salida se truncaba en el último compás y el mashup acababa
  en seco.
* **Cobertura.** Una canción nueva vale `0,3 + 0,9 × urgencia`, con urgencia = canciones que faltan / frases que
  quedan, y volver a una ya oída mientras faltan otras se cobra la mitad de eso. En la última frase cada canción que
  falta resta 0,6. Cuenta como integrada solo si suena como base o voz al menos 4 compases (una batería ajena no la
  hace reconocible).
* **Lo que no cabe.** Con menos de unos 8 s (y 4 compases) por canción no se pueden integrar todas de forma
  reconocible: el plan lo dice (`NoCaben`, con cuántas caben y cuánto haría falta) en lugar de contar
  microfragmentos.
* **Búsqueda.** Haz de 16, 10 o 6 estados según el tamaño del catálogo; candidatos por un índice barato (energía y
  coste) más la continuación natural, pistas aún sin usar y, en la primera y la última frase, aperturas y finales
  reales. Los candidatos se puntúan como tuplas ligeras y solo los que sobreviven se materializan; como mucho dos
  estados con la misma última base. El coste es lineal en frases × haz × candidatos.
* **Por qué una búsqueda por haces y no un grafo de oportunidades.** Un grafo de transiciones entre fragmentos es
  equivalente a la búsqueda por haces sobre las mismas puntuaciones, pero su tamaño crece con el cuadrado de los
  fragmentos (más de 10⁵ aristas con 80 canciones); el haz evalúa solo lo que alcanza y conserva la memoria de lo
  oído y de la cobertura en cada estado, que en un grafo serían estados aumentados.
* **Estilos.** *Equilibrado*: frases de 8 y energía en arco; la referencia de estabilidad. *Fiesta*: frases de 8 y
  una curva de energía con subida, descanso, subida más alta, respiro y clímax hacia el 86 % (luego resuelve); la
  densidad buscada es de una capa en los descansos y tres en los picos, y los coros cuentan más en el clímax.
  *Relámpago*: frases de 4, anticipo de un compás, más cortes y originales breves; los relevos de voz siguen
  obligados a respetar las pausas. *Suave*: frases de 16 y fundidos largos.
* **Degradación.** Sin candidato aceptable se prolonga la base anterior; si ni la primera frase tiene candidato, la
  petición se rechaza. Lo cedido viaja en el plan (`Degradacion`): menos canciones, no caben, sin stems, frases
  prolongadas, más corto.
* **Determinismo.** El azar (`dj::azar`, el MT19937 del DJ) solo desempata y sale de la semilla. Misma semilla,
  mismos archivos y mismas versiones: el mismo plan y el mismo WAV byte a byte (prueba).
* **Aprendizaje automático, descartado por ahora.** Se valoró un evaluador ligero entrenado con combinaciones
  originales de stems como positivos y combinaciones artificiales como negativos (la idea de Huang et al., AAAI
  2021). Sin un conjunto de evaluación perceptual propio no se puede comprobar que lo aprendido (coherencia dentro
  de una canción) mida compatibilidad entre canciones distintas, y el riesgo de aprender atajos (misma grabación,
  mismo nivel) es alto. Las medidas deterministas de arriba son explicables, no cuestan nada y no fingen una
  certeza que no hay.

## 5. El render

* Capas en orden de destino; cada una se lee una sola vez del original (solo lectura) o de la caché de stems,
  desde el primer compás de su fragmento con búsqueda precisa de Symphonia.
* Tempo por pulsos: cada pulso real del fragmento se lleva a su pulso de la rejilla del destino con un mapa a
  trozos en el vocoder, así que una batería tocada en vivo no se desliza a mitad de frase. Si faltan o sobran pulsos,
  algún tramo pide más de ±15 % o todos ya caen a menos de 8 ms, vale el tempo único del fragmento (de sus compases
  reales, acotado a ±8 %). La longitud se ajusta a la muestra.
* Igualación a −18 LUFS por capa (±12 dB como mucho) más la ganancia del papel (voz +1 dB, base −1,5 dB bajo voz,
  batería ajena −1 dB). Fundidos de igual potencia de al menos 5 ms; los cruces solapan la cola de la saliente.
* **Por bandas.** Una capa que entra filtrada o suelta los graves se parte en graves (Linkwitz-Riley de cuarto orden
  a 150 Hz) y resto, que suman exactamente la señal; cada banda lleva su envolvente y los graves cambian en un
  dieciseisavo de compás.
* **Voz al frente.** Las voces van a un acumulador aparte. Al volcar se compara la energía del acompañamiento y de la
  voz en la banda de presencia (1 a 4 kHz, filtros de segundo orden) y, solo cuando el acompañamiento la tapa (desde
  −3 dB respecto de la voz; plena a +6 dB), se atenúa esa banda del acompañamiento hasta 5 dB con ataque de 60 ms y
  relajación de 350 ms. Sin voz, o si ya se oye, la mezcla no se toca. Es sonoridad distinta de inteligibilidad:
  la igualación por capa no sabía si la base enmascaraba la voz.
* Dos pasadas: flotantes a un temporal midiendo R128; luego ganancia a −14 LUFS, un limitador de pico con 3 ms de
  anticipo (mínimo deslizante de la ganancia necesaria promediado en otra ventana igual, O(1) por muestra; techo
  −1 dBFS), el limitador suave del motor y WAV de 16 bits, con métricas (sonoridad, pico, pico verdadero
  sobremuestreado de EBU R128, recortes, salto máximo entre muestras, hueco de silencio más largo).
* El audio dura el plan más la cola de las capas que terminan en su último compás (hasta cuatro compases).
* Los temporales se borran si algo falla o se cancela; el WAV aparece por renombrado atómico. Al arrancar, lo que la
  base no conoce en `datos/mashups/` se borra y lo que quedó `generando` pasa a `fallida`.

## 6. Stems y cachés

| Qué | Dónde | Clave | Vida |
|---|---|---|---|
| Instrumental del karaoke | `datos/stems/` | pista | Del usuario (ADR-006); no cambia |
| Cuatro stems, enteros o de un tramo | `cache/stems-mashup/<sha>-htdemucs-onnx-fp16-v1/` | hash, modelo, versión | LRU con el tope de Ajustes (5 GB de fábrica) |
| Estructura (versión 2: con voz por cuartos y patrón rítmico) | `estructura_pistas` | pista, hash, versión | Caduca con el archivo o el algoritmo |
| Mashup | `mashups`, `mashup_pistas`, `datos/mashups/<id>.wav` | id | Del usuario |

Un archivo que cambia tiene otro hash y otra entrada; la vieja la recoge la poda. Si la cola del karaoke esperaba
una pista que el mashup separa, se completa con esa separación (mezcla menos voz) sin volver a pasar el modelo.

* **Una caché para todo.** El karaoke separa los cuatro stems (el modelo los calcula de todos modos), los guarda
  aquí y saca el instrumental de ellos; si la canción entera ya está, no pasa el modelo. Así cada canción se separa
  una vez para el karaoke y los mashups. Leído de la caché, el instrumental difiere del de siempre en menos de una
  unidad de 16 bits.
* **Solo lo que suena como stems.** El plan previo supone stems para todas las candidatas; lo que usa solo con su
  mezcla original no se separa. Sin separador se usa lo que ya está en la caché.
* **Tramos.** Un mashup separa solo los compases que usa su plan previo, con una frase de margen a cada lado, dos
  compases de cola y dos segundos de contexto para el modelo; la entrada guarda su primera y última muestra
  (`tramo`). El plan definitivo solo elige fragmentos dentro de lo separado. Si el tramo pedido no está, se separa
  la envolvente del viejo y del nuevo; si pasa del 85 % de la canción, la canción entera, que sirve al karaoke.
* **Hilos.** Un cuarto de los núcleos, con prioridad baja. Más no acelera: con una canción de 164 s, 2 hilos
  separan a 0,38 veces el tiempo real y 4 a 0,37 (2026-11-15). Lo que acelera es separar menos audio.
* **Protección.** Lo que usa una generación en curso no se poda aunque pode el karaoke a la vez.

## 7. Interfaz

`VistaDj.svelte` monta las dos experiencias en el mismo hueco y un solo `HistorialDj`. Cada experiencia se monta la
primera vez que se abre y después solo se oculta: lo escrito sigue ahí. Las reglas (qué se ve al entrar, qué marca el
historial, carriles y cajas de la línea de tiempo) están en `ui/dj/experiencias.ts` con sus pruebas.

El paquete de arranque solo creció lo que la barra necesita para un mashup (el título y habilitar sus controles); el
resto va en el trozo del DJ. Para no pasar del presupuesto de 250 KB, el mashup usa iconos del catálogo
(`ecualizador`) en lugar de uno nuevo y la instantánea solo lleva `mashupId` cuando suena uno.

## 8. Pruebas

Cada módulo tiene las suyas en `src-tauri/src/mashup/`: separación idéntica a la del karaoke, caché que reutiliza,
invalida por huella y no pasa del tope, análisis estructural sobre señales sintéticas (tempo ±1 BPM, pulsos ±20 ms),
actividad vocal que distingue canto de residuo, estiramiento que conserva el tono (±5 cents), la duración y los
formantes, render sin recortes ni clics a −14 LUFS ±1 y limitador con anticipo que deja el pico en su techo sin
escalones, y persistencia que sobrevive a cancelar y reabrir.

El compositor se prueba con bibliotecas sintéticas, que validan invariantes y no calidad perceptual: 16 canciones
reconocibles de 80 con tonalidades de toda la rueda en los tres estilos principales; canciones con armonías
incompatibles que se integran en secuencia, con cortes y sin voces forzadas; armonía compás a compás que manda
sobre una media favorable; clave 3-2 contra 2-3 detectada, en fase al desplazar un compás, nula con un bombo a
negras y atenuada con una rejilla dudosa; ninguna voz sobre una base en clave opuesta sin perder canciones; relevos
de voz en las pausas; colas de voz al soltarla; aperturas por un principio y cierres sin corte seco; voces casi
nunca transpuestas y el original sin dominar; estilos distintos (Relámpago cambia más, Fiesta tiene descansos y el
clímax entre el 75 y el 95 %); "no caben" explicado; reproducibilidad por semilla; sin repetir material mientras
quede nuevo, también el de los anticipos. `mashups_reales_desde_una_copia_de_los_datos` (ignorada) genera mashups
con la biblioteca y la caché reales sin tocarlas. Las mediciones están en la sección 9.

## 9. Mediciones

Equipo de desarrollo (Linux, CPU, sin GPU para ONNX), compilación optimizada, 2026-11-14. Se repiten con
`cargo test --release --lib mashup::rendimiento -- --ignored --test-threads=1 --nocapture` (variables en el propio
módulo) y `scripts/qa/ui-qa.sh fase10`.

| Medición | Resultado |
|---|---|
| Análisis estructural, 20 canciones reales (67 min) | 0,50 s de CPU por minuto de audio; rejilla fiable en 17 de 20; 193 MB de pico |
| Compositor, 5 / 10 / 20 / 50 / 100 canciones, 2026-11-18 | 64 / 43 / 140 / 657 / 1.630 ms (antes 65 / 22 / 70 / 406 / 1.103); canciones usadas 5 / 10 / 20 / 60 / 108 de las pedidas (antes 4 / 7 / 14 / 54 / 108) |
| Vocoder (30 s, +5 % y +1 semitono), 2026-11-18 | 53 veces más rápido que el tiempo real (antes 55) |
| Render sintético, 2 / 6 / 15 min, 2026-11-18 | 14× / 24× / 25× el tiempo real (antes 47× / 43× / 34×); acumulador máximo 34 / 37 / 47 s; 92 / 111 / 115 MB de pico; −14,0 LUFS; pico −1,0 dBFS (antes −3,1 a −1,8 sin control). La diferencia viene de que los planes nuevos pasan más capas por el vocoder (otro tempo de referencia y transposiciones de base), no del procesado añadido: en el de 2 min las capas suman 3,9 de 4,6 s y la segunda pasada 0,4 s |
| Mashups reales con la caché real (copia de los datos, sin separar), 2026-11-18 | Equilibrado 6 canciones / 3 min: 8 reconocibles, 7 voces, 7 entradas con intercambio de graves y 1 corte, 34 s en total con el análisis de 83 candidatas; Fiesta 8 / 4 min: 11, con 10 baterías ajenas, 19 s; Relámpago 12 / 3 min: 13, 18 s; 16 de 80 en 6 min: 18 reconocibles, 4 de 14 voces transpuestas, 42 s; cuatro salsas de Tito Rojas: 4 de 4, 6 s; 30 canciones en 1 min: "caben 10, harían falta 147 s". Todos a −14,0 LUFS ±0,03, sin recortes, pico −1,0 dBFS, pico verdadero entre −0,9 y −0,2 dBTP, 370 MB de pico |
| Separación real de 4 stems (2 min 44 s) | 0,54× el tiempo real (303 s); 1,33 GB de pico; la suma de stems reconstruye la mezcla a 34 dB |
| Caché FLAC | 17,3 MB por minuto de canción (los cuatro stems) |
| Mashup real (12 candidatas, 2 min) | 1.008 s la primera vez (casi todo separación); 4,2 s con caché; −14,0 LUFS, sin recortes |
| Separación real, 2026-11-15, 2 y 4 hilos | 0,38× y 0,37× el tiempo real: más hilos no aceleran |
| Mashup real con tramos, misma petición | 616 s la primera vez (1.002 s sin tramos), 73 MB de caché en vez de 170; 4,8 s con caché; −14,0 LUFS, sin recortes ni silencios |
| Tramos con media frase de margen | 568 s, pero el plan definitivo tuvo que prolongar una frase: se descartó |
| App real, mashup de 4 canciones y 3 min | 33 min con separación, sin cortar la canción que sonaba (612 s reproducidos durante la generación); el reproductor respondió en 8 ms de media y 24 ms como máximo; progreso siempre creciente |
| Cancelar en la app real | 1,3 s hasta `cancelada`; generador libre y sin archivos |
| Alternar experiencias en la app real | 37 ms; el borrador y el historial se conservan |

## 10. Límites conocidos

* **La separación domina.** HTDemucs en CPU va a 0,5× el tiempo real: un mashup de 4 canciones nuevas tarda
  unos 15 a 35 minutos la primera vez; de 30 canciones, horas. La caché lo vuelve segundos. La aceleración por GPU
  del karaoke se aplica sin cambios, pero no se ha medido aquí.
* **La calidad de 50 a 100 canciones** está medida en forma (todas entran, frases de 4 compases, sin repetir), no
  en escucha. La puntuación media por frase satura en 1,0 con material sintético y no sirve para comparar planes.
* **El estiramiento** está probado en tono (±5 cents), duración (±1 muestra), golpes y formantes de una vocal
  sintética; los artefactos en voces reales a ±8 % y ±2 semitonos no se han evaluado de oído. La corrección de
  formantes no arregla los artefactos de la separación (sangrado de la batería en la voz).
* **Calidad perceptual sin medir.** Las pruebas validan invariantes y las mediciones reales forma y sonoridad; que
  las transiciones, los relevos y los cierres suenen mejor solo se puede comprobar escuchando. Ninguna puntuación
  del compositor se presenta como calidad.
* **Clave.** La comparación de fases de dos compases se ha validado con patrones sintéticos de son; no hay un corpus
  de salsa etiquetado para medir aciertos (Salsa, TISMIR 2024, muestra que incluso el pulso es difícil en este
  género). Con una rejilla dudosa, el conflicto pesa poco: el sistema prefiere cortes a afirmar una fase.
* **Equilibrio de la voz.** La atenuación de la banda de presencia actúa solo donde el acompañamiento tapa a la voz
  en energía; no modela el enmascaramiento psicoacústico por bandas críticas.
* **Compás 4/4.** Las piezas en 3/4 o con cambios de compás quedan con confianza baja y se alinean peor.
* **Picos.** El limitador con anticipo trabaja sobre muestras a −1 dBFS; el pico verdadero se mide pero no se
  limita sobremuestreando: en las mezclas reales quedó entre −0,9 y −0,2 dBTP.
* **Huecos.** En el mashup real de la app hubo un tramo de 250 ms por debajo de −60 dBFS; viene de la música de
  origen o de un fundido sobre un silencio del fragmento, no de un hueco del render (el sintético no tiene ninguno).
* **Selección manual.** La interfaz deja elegir la biblioteca o una playlist, no canciones sueltas; el puente ya
  acepta una lista de pistas.

## 11. Referencias

Métodos establecidos que se usan: novedad sobre autosimilitud (Foote, 2000); bloqueo de fase de identidad (Laroche y
Dolson, 1999); envolvente cepstral para conservar formantes (Röbel y Rodet, 2005); patrones de inicio por
subdivisión métrica para comparar ritmos; Linkwitz-Riley para partir bandas; EBU R128 para sonoridad y pico
verdadero. Adaptaciones propias: la elección de transición por armonía y estructura, la fase relativa de dos
compases con su confianza, la cola vocal y la presión de cobertura del haz.

Trabajos que orientaron las decisiones, sin copiar su código:

* Mosaikbox (ISMIR 2024, TU Wien): quitar o atenuar capas que no encajan en lugar de forzarlas; aquí, la mezcla
  original, los cortes estructurales y una sola batería.
* Huang et al., *Modeling the Compatibility of Stem Tracks to Generate Music Mashups* (AAAI 2021): compatibilidad
  por papel de cada stem; el evaluador aprendido se descartó (sección 4).
* AutoMashup (2025, arXiv 2508.06516): la compatibilidad es asimétrica; aquí cada pareja se evalúa como voz de una
  sobre base de otra, en los dos sentidos por separado.
* *Salsa, a Dataset for Beat Estimation in Salsa Music* (TISMIR 2024, con su corrección): las dificultades del pulso
  en salsa; por eso la clave se trata con confianza y sin clasificador.
