-- V13: DJ Mashups (ADR-018).
--
-- La definición de un mashup (petición, semilla, plan) es del usuario y va en la
-- base; su audio es derivado y se puede regenerar desde el plan. Los stems y la
-- estructura son cachés: la estructura vive aquí porque es pequeña y se consulta
-- por pista; los stems, en la caché de disco.

-- La estructura temporal de una pista. Caduca si cambia el archivo (sha256) o
-- el algoritmo (version).
CREATE TABLE estructura_pistas (
    pista_id    INTEGER PRIMARY KEY REFERENCES pistas (id) ON DELETE CASCADE,
    sha256      TEXT    NOT NULL CHECK (length(sha256) = 64),
    version     INTEGER NOT NULL CHECK (version > 0),
    datos       TEXT    NOT NULL CHECK (json_valid(datos)),
    -- 1 si `datos` incluye la actividad vocal sacada del stem de voz.
    con_voz     INTEGER NOT NULL DEFAULT 0 CHECK (con_voz IN (0, 1)),
    calculada_en TEXT   NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE TABLE mashups (
    id                INTEGER PRIMARY KEY,
    titulo            TEXT    NOT NULL CHECK (titulo <> ''),
    -- JSON: duración, canciones, estilo, semilla y las pistas elegidas, si las hubo.
    peticion          TEXT    NOT NULL CHECK (json_valid(peticion)),
    semilla           INTEGER NOT NULL,
    version           INTEGER NOT NULL CHECK (version > 0),
    estado            TEXT    NOT NULL DEFAULT 'generando'
                              CHECK (estado IN ('generando', 'lista', 'fallida', 'cancelada')),
    -- JSON del plan: todo lo necesario para volver a renderizarlo.
    plan              TEXT    CHECK (plan IS NULL OR json_valid(plan)),
    -- Relativo a la carpeta de mashups de NB Sound.
    archivo           TEXT,
    duracion_seg      REAL    CHECK (duracion_seg IS NULL OR duracion_seg >= 0),
    bytes             INTEGER CHECK (bytes IS NULL OR bytes >= 0),
    metricas          TEXT    CHECK (metricas IS NULL OR json_valid(metricas)),
    error             TEXT,
    creado_en         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    terminado_en      TEXT,
    -- Una lista tiene su archivo y su plan.
    CHECK ((estado = 'lista') = (archivo IS NOT NULL AND plan IS NOT NULL))
) STRICT;

CREATE INDEX mashups_por_fecha ON mashups (creado_en);

-- Las pistas que suenan en un mashup, para buscar y para saber qué stems
-- protege. Borrar una pista no borra el mashup ya renderizado: su audio sigue
-- siendo válido, solo deja de poder regenerarse.
CREATE TABLE mashup_pistas (
    mashup_id  INTEGER NOT NULL REFERENCES mashups (id) ON DELETE CASCADE,
    orden      INTEGER NOT NULL CHECK (orden >= 0),
    pista_id   INTEGER REFERENCES pistas (id) ON DELETE SET NULL,
    titulo     TEXT    NOT NULL,
    artista    TEXT    NOT NULL,
    PRIMARY KEY (mashup_id, orden)
) STRICT, WITHOUT ROWID;

CREATE INDEX mashup_pistas_por_pista ON mashup_pistas (pista_id);
