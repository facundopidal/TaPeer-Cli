mod client;
mod models;

use anyhow::{Context, Result, bail};
use models::ItemType;
use tokio::fs;

use colored::Colorize;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use client::TaPeerClient;

#[derive(Parser, Debug)]
#[command(
    name = "tapeer",
    author,
    version,
    about = "CLI rápida para compartir archivos y texto entre dispositivos con TaPeer",
    long_about = "TaPeer CLI te permite enviar y recibir archivos y notas de texto a través de tu red local o Tailscale
sin configuraciones complejas.",
    after_help = "EJEMPLOS DE USO:
  tapeer list                         Ver los últimos elementos compartidos
  tapeer text \"clave wifi: 1234\"      Enviar una nota de texto
  tapeer send ./documento.pdf         Enviar un archivo
  tapeer get                          Descargar o ver el elemento más reciente
  tapeer get 1                        Descargar el elemento #1 de la lista
  tapeer get 2 -c                     Copiar el texto #2 directo al portapapeles
  tapeer get foto.png -o descargas/   Descargar 'foto.png' en la carpeta descargas/"
)]
struct Cli {
    /// URL del servidor TaPeer (usa TAPEER_URL si está definida)
    #[arg(
        short,
        long,
        env = "TAPEER_URL",
        default_value = "http://localhost:3000"
    )]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Lista los elementos compartidos activos en TaPeer
    List,

    /// Comparte un texto, enlace o nota rápida
    Text(TextArgs),

    /// Envía un archivo desde tu disco a TaPeer
    Send(SendArgs),

    /// Descarga un archivo o muestra un texto (por número, nombre, ID o el más reciente)
    Get(GetArgs),
}

#[derive(Args, Debug)]
struct TextArgs {
    /// El texto o mensaje que deseas compartir
    #[arg(help = "Texto a compartir (usa comillas si contiene espacios)")]
    text: String,
}

#[derive(Args, Debug)]
struct SendArgs {
    /// Ruta del archivo a enviar
    #[arg(help = "Ruta al archivo que deseas subir (ej: ./foto.png)")]
    path: PathBuf,
}

#[derive(Args, Debug)]
struct GetArgs {
    /// Selector: número de lista [1..N], nombre de archivo, prefijo de ID, o vacío para el más reciente
    #[arg(help = "Identificador del elemento (1, 2, nombre, o vacío para el último)")]
    target: Option<String>,

    /// Ruta o carpeta donde guardar el archivo descargado
    #[arg(short, long, help = "Archivo o carpeta de destino (ej: -o descargas/)")]
    output: Option<PathBuf>,

    /// Copia el contenido del texto al portapapeles
    #[arg(short, long, help = "Copia el texto (o ruta) directo al portapapeles")]
    copy: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let client = TaPeerClient::new(args.server);

    match &args.command {
        Commands::List => {
            let items = client.list_items().await?;

            if items.is_empty() {
                println!(
                    "{}",
                    "No hay elementos compartidos en TaPeer actualmente.".dimmed()
                );
                return Ok(());
            }

            println!(
                "\n{} {}\n",
                " TaPeer ".on_green().black().bold(),
                format!("({} elementos activos)", items.len()).dimmed()
            );

            for (idx, item) in items.iter().enumerate() {
                let (icon, badge) = match item.item_type {
                    ItemType::File => ("📦", "Archivo".blue().bold()),
                    ItemType::Text => ("📝", "Texto  ".yellow().bold()),
                };

                let size_str = format_size(item.size).cyan();
                let expiry_str = format_expiry(item.expiry_time).dimmed();
                let num = format!("#{}", idx + 1).bold();

                println!(
                    "  {:>3} {} [{}] {}  •  {}  •  {}",
                    num,
                    icon,
                    badge,
                    item.original_name.bold(),
                    size_str,
                    expiry_str
                );

                if let Some(content) = &item.content {
                    let preview = if content.len() > 60 {
                        format!("{}...", &content[..60])
                    } else {
                        content.clone()
                    };
                    println!("      {} \"{}\"", "↳".dimmed(), preview.italic());
                } else {
                    println!("      {} id: {}", "↳".dimmed(), item.id.dimmed());
                }

                println!();
            }

            println!(
                "{}\n",
                "Tip: Usa 'tapeer get 1' para descargar o 'tapeer get 1 -c' para copiar al portapapeles."
                    .dimmed()
                    .italic()
            );
        }
        Commands::Send(send_args) => {
            println!(
                "{} Subiendo '{}' a TaPeer...",
                "↑".cyan().bold(),
                send_args.path.display().to_string().bold()
            );
            let res = client.upload_file(&send_args.path).await?;

            println!(
                "{} {}",
                "✓".green().bold(),
                "Archivo subido con éxito".bold()
            );
            println!("  {} {}", "Nombre:".dimmed(), res.file_name.bold());
            println!("  {} {}", "ID:".dimmed(), res.id.cyan());
            println!(
                "  {} {}{}",
                "URL:".dimmed(),
                client.base_url(),
                res.download_url
            );
        }
        Commands::Text(text_args) => {
            let res = client.share_text(&text_args.text).await?;

            println!(
                "{} {}",
                "✓".green().bold(),
                "Texto compartido con éxito".bold()
            );
            println!("  {} {}", "ID:".dimmed(), res.id.cyan());
            println!(
                "  {} {}{}",
                "URL:".dimmed(),
                client.base_url(),
                res.snippet_url
            );
        }
        Commands::Get(get_args) => {
            let items = client.list_items().await?;
            let item = resolve_target(&items, get_args.target.as_deref())?;

            match item.item_type {
                ItemType::Text => {
                    let content = match &item.content {
                        Some(c) => c.clone(),
                        None => client.get_snippet(&item.id).await?,
                    };

                    // Si se pidió copiar al portapapeles
                    if get_args.copy {
                        let mut clipboard = arboard::Clipboard::new()
                            .context("No se pudo acceder al portapapeles del sistema")?;
                        clipboard
                            .set_text(&content)
                            .context("No se pudo copiar el texto al portapapeles")?;
                        println!("✓ Texto copiado al portapapeles.");
                    }

                    // Si se especificó un destino, se guarda en archivo
                    if let Some(output_path) = &get_args.output {
                        let dest = if output_path.is_dir()
                            || output_path.to_string_lossy().ends_with('/')
                            || output_path.to_string_lossy().ends_with('\\')
                        {
                            output_path.join(format!("{}.txt", &item.id[..8]))
                        } else {
                            output_path.clone()
                        };

                        if let Some(parent) = dest.parent() {
                            if !parent.as_os_str().is_empty() {
                                fs::create_dir_all(parent).await?;
                            }
                        }

                        fs::write(&dest, content.as_bytes()).await?;
                        println!("✓ Texto guardado en: {}", dest.display());
                    } else if !get_args.copy {
                        // Si no se guardó en archivo ni se pidió solo copiar, se imprime en pantalla
                        println!("{}", content);
                    }
                }

                ItemType::File => {
                    // Detección inteligente de destino: si es carpeta o archivo
                    let destination = match &get_args.output {
                        Some(out) => {
                            let is_folder = out.is_dir()
                                || out.to_string_lossy().ends_with('/')
                                || out.to_string_lossy().ends_with('\\');

                            if is_folder {
                                out.join(&item.original_name)
                            } else {
                                out.clone()
                            }
                        }
                        None => PathBuf::from(&item.original_name),
                    };

                    // Crear carpetas padre si no existen todavía (ej: si pasas --output descargas/)
                    if let Some(parent) = destination.parent() {
                        if !parent.as_os_str().is_empty() {
                            fs::create_dir_all(parent).await.context(format!(
                                "No se pudo crear el directorio '{}'",
                                parent.display()
                            ))?;
                        }
                    }

                    println!(
                        "{} Descargando '{}'...",
                        "↓".cyan().bold(),
                        item.original_name.bold()
                    );
                    let bytes = client.download_file(&item.id).await?;

                    fs::write(&destination, &bytes).await?;
                    println!(
                        "{} Archivo guardado con éxito en: {}",
                        "✓".green().bold(),
                        destination.display().to_string().bold()
                    );

                    // Si pasó --copy con un archivo, copiamos la ruta del archivo descargado al portapapeles
                    if get_args.copy {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let path_str = destination.display().to_string();
                            let _ = clipboard.set_text(path_str);
                            println!("{} Texto copiado al portapapeles.", "✓".green().bold());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

use models::SharedItem;

/// Busca un item en la lista según el criterio ingresado por el usuario
fn resolve_target<'a>(items: &'a [SharedItem], target: Option<&str>) -> Result<&'a SharedItem> {
    if items.is_empty() {
        bail!("No hay elementos compartidos disponibles en TaPeer.");
    }

    // 1. Si no especificó nada, o escribió "latest", tomamos el más reciente (posición 0)
    let query = match target {
        None => return Ok(&items[0]),
        Some("latest") => return Ok(&items[0]),
        Some(q) => q.trim(),
    };

    // 2. Si es un número (ej: 1, 2, 3), buscamos por índice de lista
    if let Ok(index) = query.parse::<usize>() {
        if index >= 1 && index <= items.len() {
            return Ok(&items[index - 1]);
        } else {
            bail!(
                "Índice '{}' fuera de rango. Hay {} elementos disponibles (1 a {}).",
                index,
                items.len(),
                items.len()
            );
        }
    }

    // 3. Si coincide con el UUID o con el inicio del UUID (ej: "c1f7")
    if let Some(item) = items.iter().find(|it| it.id.starts_with(query)) {
        return Ok(item);
    }

    // 4. Si coincide con el nombre del archivo (búsqueda parcial insensible a mayúsculas)
    let query_lower = query.to_lowercase();
    let matches: Vec<&SharedItem> = items
        .iter()
        .filter(|it| it.original_name.to_lowercase().contains(&query_lower))
        .collect();

    match matches.len() {
        1 => Ok(matches[0]),
        count if count > 1 => {
            bail!(
                "Hay {} elementos que coinciden con '{}'. Sé más específico o usa el número de índice.",
                count,
                query
            );
        }
        _ => bail!(
            "No se encontró ningún elemento que coincida con '{}'.",
            query
        ),
    }
}
/// Formatea bytes a un formato legible por humanos (B, KB, MB, GB)
fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

/// Calcula el tiempo restante aproximado hasta la expiración
fn format_expiry(expiry_timestamp_ms: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    if expiry_timestamp_ms <= now_ms {
        return "Expirado".to_string();
    }

    let diff_secs = (expiry_timestamp_ms - now_ms) / 1000;
    let hours = diff_secs / 3600;
    let minutes = (diff_secs % 3600) / 60;

    if hours > 0 {
        format!("expira en {}h {}m", hours, minutes)
    } else {
        format!("expira en {}m", minutes)
    }
}
