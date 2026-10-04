#[cfg(not(target_os = "linux"))]
use anyhow::{Context, Result};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(any(target_os = "linux", test))]
mod protocol;

#[cfg(target_os = "linux")]
pub use linux::{copy_text, is_helper_invocation, read_text, run_helper};

#[cfg(not(target_os = "linux"))]
pub fn copy_text(text: &str) -> Result<()> {
    arboard::Clipboard::new()
        .context("No se pudo acceder al portapapeles del sistema")?
        .set_text(text)
        .context("No se pudo copiar el texto al portapapeles")
}

#[cfg(not(target_os = "linux"))]
pub fn read_text() -> Result<String> {
    arboard::Clipboard::new()
        .context("No se pudo acceder al portapapeles del sistema")?
        .get_text()
        .context("El portapapeles no contiene texto válido o no está disponible")
}
