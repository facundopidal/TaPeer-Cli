use serde::{Deserialize, Serialize};

/// Tipo de elemento compartido en TaPeer: o es un archivo o es un texto.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    File,
    Text,
}

/// Representa cada elemento que devuelve el endpoint GET /items.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedItem {
    pub id: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: u64,
    pub upload_time: u64,
    pub expiry_time: u64,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    /// Solo viene cuando el item es de tipo texto. Si es un archivo, viene nulo o ausente.
    pub content: Option<String>,
}

/// Lo que le enviamos a TaPeer en POST /text para compartir texto.
#[derive(Debug, Serialize)]
pub struct TextPayload {
    pub text: String,
}

/// La respuesta que nos devuelve TaPeer al compartir texto (POST /text).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextResponse {
    pub id: String,
    pub snippet_url: String,
    pub expiry_time: u64,
}

/// La respuesta que nos devuelve TaPeer al subir un archivo (POST /upload).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    pub id: String,
    pub file_name: String,
    pub download_url: String,
    pub expiry_time: u64,
}
