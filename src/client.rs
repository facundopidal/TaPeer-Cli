use reqwest::multipart::{Form, Part};
use std::path::Path;
use tokio::fs;

use anyhow::{Context, Result};
use reqwest::Client;

use crate::models::{SharedItem, TextPayload, TextResponse, UploadResponse};

pub struct TaPeerClient {
    base_url: String,
    http: Client,
}
impl TaPeerClient {
    /// Crea una nueva instancia del cliente.
    /// Normaliza la URL quitando cualquier '/' final para evitar errores al concatenar rutas.
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: Client::new(),
        }
    }
    /// Llama a GET /items y devuelve la lista de elementos compartidos activos.
    pub async fn list_items(&self) -> Result<Vec<SharedItem>> {
        let url = format!("{}/items", self.base_url);

        let response = self.http.get(&url).send().await.context(format!(
            "No se pudo conectar con el servidor en '{}'",
            self.base_url
        ))?;

        // Verifica si el servidor devolvió un código 4xx o 5xx
        let response = response
            .error_for_status()
            .context("El servidor TaPeer respondió con un error HTTP")?;

        // Deserializa automáticamente el cuerpo JSON en un Vec<SharedItem>
        let items = response
            .json::<Vec<SharedItem>>()
            .await
            .context("No se pudo interpretar la respuesta JSON del servidor")?;

        Ok(items)
    }
    /// Getter para consultar la URL base configurada
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Llama a POST /text enviando el texto en formato JSON
    pub async fn share_text(&self, text: &str) -> Result<TextResponse> {
        let url = format!("{}/text", self.base_url);
        let payload = TextPayload {
            text: text.to_string(),
        };

        let response = self
            .http
            .post(&url)
            .json(&payload) // Serializa a JSON y agrega el header 'Content-Type: application/json'
            .send()
            .await
            .context(format!(
                "No se pudo conectar con el servidor en '{}'",
                self.base_url
            ))?;

        let response = response
            .error_for_status()
            .context("El servidor TaPeer rechazó el texto enviado")?;

        let result = response
            .json::<TextResponse>()
            .await
            .context("No se pudo interpretar la respuesta del servidor al compartir texto")?;

        Ok(result)
    }
    /// Sube un archivo a TaPeer llamando a POST /upload con multipart/form-data
    pub async fn upload_file(&self, file_path: &Path) -> Result<UploadResponse> {
        // 1. Extraemos el nombre del archivo desde la ruta
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .context("No se pudo obtener un nombre de archivo válido de la ruta proporcionada")?
            .to_string();

        // 2. Leemos los bytes del archivo desde el disco de forma asíncrona
        let file_bytes = fs::read(file_path).await.context(format!(
            "No se pudo leer el archivo '{}'",
            file_path.display()
        ))?;

        // 3. Construimos el formulario multipart con el campo 'file'
        let part = Part::bytes(file_bytes).file_name(file_name);
        let form = Form::new().part("file", part);

        let url = format!("{}/upload", self.base_url);

        // 4. Enviamos la petición POST
        let response = self
            .http
            .post(&url)
            .multipart(form)
            .send()
            .await
            .context(format!(
                "No se pudo conectar con el servidor en '{}'",
                self.base_url
            ))?;

        let response = response
            .error_for_status()
            .context("El servidor TaPeer rechazó el archivo subido")?;

        let result = response
            .json::<UploadResponse>()
            .await
            .context("No se pudo interpretar la respuesta del servidor al subir el archivo")?;

        Ok(result)
    }
    /// Descarga los bytes de un archivo desde GET /download/:id
    pub async fn download_file(&self, id: &str) -> Result<Vec<u8>> {
        let url = format!("{}/download/{}", self.base_url, id);
        let response = self.http.get(&url).send().await.context(format!(
            "No se pudo conectar con el servidor en '{}'",
            self.base_url
        ))?;

        let response = response
            .error_for_status()
            .context("El servidor TaPeer no encontró el archivo o ya expiró")?;

        let bytes = response
            .bytes()
            .await
            .context("Error al descargar los datos del archivo")?;

        Ok(bytes.to_vec())
    }

    /// Obtiene el contenido de un snippet de texto desde GET /snippet/:id
    pub async fn get_snippet(&self, id: &str) -> Result<String> {
        let url = format!("{}/snippet/{}", self.base_url, id);
        let response = self.http.get(&url).send().await.context(format!(
            "No se pudo conectar con el servidor en '{}'",
            self.base_url
        ))?;

        let response = response
            .error_for_status()
            .context("El servidor TaPeer no encontró el texto o ya expiró")?;

        let text = response
            .text()
            .await
            .context("Error al leer el texto de la respuesta")?;

        Ok(text)
    }
}
