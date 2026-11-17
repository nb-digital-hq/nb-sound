//! DJ Mashups (ADR-018): componer mashups con fragmentos de la
//! biblioteca.
//!
//! El recorrido de una generación, de lo barato a lo caro:
//! 1. [`seleccion`]: prefiltro por los rasgos ya guardados (tempo, tonalidad,
//!    energía) para no analizar pistas que nunca encajarían.
//! 2. [`estructura`]: pulsos, compases y secciones de las preseleccionadas,
//!    guardados por hash y versión.
//! 3. [`stems`]: los cuatro stems con el separador del karaoke, en caché.
//! 4. [`compositor`]: el plan, por búsqueda por haces.
//! 5. [`render`]: el audio, con [`estiramiento`] para tempo y tono.
//!
//! [`servicio`] orquesta los pasos en un hilo de fondo y [`persistencia`] guarda
//! la definición del mashup, que es lo que permite regenerarlo.

pub mod compositor;
pub mod estiramiento;
pub mod estructura;
pub mod persistencia;
pub mod render;
#[cfg(test)]
mod rendimiento;
pub mod seleccion;
pub mod servicio;
pub mod stems;
