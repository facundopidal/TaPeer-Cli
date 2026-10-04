use std::io::{self, Read};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};

use super::protocol;

const HELPER: &str = "--clipboard-helper";
const SETUP_TIMEOUT: Duration = Duration::from_secs(5);

pub fn is_helper_invocation() -> bool {
    let mut args = std::env::args_os().skip(1);
    args.next().as_deref() == Some(std::ffi::OsStr::new(HELPER)) && args.next().is_none()
}

fn use_wayland() -> bool {
    ["WAYLAND_DISPLAY", "WAYLAND_SOCKET"]
        .iter()
        .any(|key| std::env::var_os(key).is_some_and(|value| !value.is_empty()))
}

pub fn read_text() -> Result<String> {
    if use_wayland() {
        use wl_clipboard_rs::paste::{ClipboardType, MimeType, Seat, get_contents};
        let (mut pipe, _) = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text)
            .context("El portapapeles Wayland no contiene texto o no está disponible")?;
        let mut text = String::new();
        pipe.read_to_string(&mut text)
            .context("No se pudo leer texto UTF-8 del portapapeles Wayland")?;
        Ok(text)
    } else {
        arboard::Clipboard::new()
            .context("No se pudo acceder al portapapeles X11")?
            .get_text()
            .context("El portapapeles X11 no contiene texto o no está disponible")
    }
}

// On any setup failure, terminate and reap the helper before joining its pipe worker.
struct PendingHelper(Option<Child>);

impl Drop for PendingHelper {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn copy_text(text: &str) -> Result<()> {
    let mut command = Command::new(std::env::current_exe()?);
    command
        .arg(HELPER)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command
        .spawn()
        .context("No se pudo iniciar el proceso de portapapeles")?;
    let mut input = child.stdin.take().context("Falta el canal de entrada")?;
    let mut output = child
        .stdout
        .take()
        .context("Falta el canal de confirmación")?;
    let mut pending = PendingHelper(Some(child));
    let text = text.to_owned();
    let (sender, receiver) = mpsc::sync_channel(1);
    let worker = thread::Builder::new().spawn(move || {
        let result = (|| {
            protocol::write_request(&mut input, &text)?;
            protocol::read_status(&mut output)?;
            protocol::write_accept(&mut input)
        })();
        let _ = sender.send(result);
    })?;
    let result = receiver
        .recv_timeout(SETUP_TIMEOUT)
        .context("El portapapeles no confirmó la copia en 5 segundos")
        .and_then(|result| result);
    if result.is_err() {
        drop(pending);
        let _ = worker.join();
        return result;
    }
    worker
        .join()
        .map_err(|_| anyhow::anyhow!("Falló el canal de portapapeles"))?;
    // Reap if replacement happens while the foreground CLI is still running.
    // Dropping this thread's handle does not keep the foreground process alive.
    thread::Builder::new().spawn(move || {
        if let Some(mut child) = pending.0.take() {
            let _ = child.wait();
        }
    })?;
    Ok(())
}

pub fn run_helper() -> Result<()> {
    // This watchdog bounds startup only. Acceptance cancels it permanently;
    // clipboard lifetime is driven by ownership/session events, not elapsed time.
    let (accepted, watchdog) = mpsc::sync_channel(1);
    thread::spawn(move || {
        if watchdog
            .recv_timeout(SETUP_TIMEOUT + Duration::from_secs(1))
            .is_err()
        {
            std::process::exit(1);
        }
    });
    let result = (|| {
        rustix::process::setsid().context("No se pudo separar el proceso de portapapeles")?;
        let mut input = io::stdin();
        let text = protocol::read_request(&mut input)?;
        let confirm = move || {
            protocol::write_status(io::stdout(), true)?;
            protocol::read_accept(input)?;
            accepted.send(())?;
            Ok(())
        };
        if use_wayland() {
            serve_wayland(text, confirm)
        } else {
            serve_x11(text, confirm)
        }
    })();
    if result.is_err() {
        // Fixed protocol errors only: never print clipboard bytes or backend debug output.
        let _ = protocol::write_status(io::stdout(), false);
    }
    // Avoid Rust's main Result diagnostics on the helper's inherited descriptors.
    std::process::exit(if result.is_ok() { 0 } else { 1 });
}

fn serve_wayland(
    text: String,
    confirm: impl FnOnce() -> Result<()> + Send + 'static,
) -> Result<()> {
    use wl_clipboard_rs::{copy, paste};
    let marker = format!(
        "application/x-tapeer-ready-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    );
    let mut options = copy::Options::new();
    options.foreground(true);
    let prepared = options.prepare_copy_multi(vec![
        copy::MimeSource {
            source: copy::Source::Bytes(text.as_bytes().into()),
            mime_type: copy::MimeType::Text,
        },
        copy::MimeSource {
            source: copy::Source::Bytes(marker.as_bytes().into()),
            mime_type: copy::MimeType::Specific(marker.clone()),
        },
    ])?;
    // prepare_copy only queues set_selection. Readiness requires a request that
    // this unique live source actually serves, not merely successful preparation.
    thread::Builder::new().spawn(move || {
        let result = (|| {
            loop {
                match paste::get_contents(
                    paste::ClipboardType::Regular,
                    paste::Seat::Unspecified,
                    paste::MimeType::Specific(&marker),
                ) {
                    Ok((mut pipe, _)) => {
                        let mut bytes = Vec::new();
                        pipe.read_to_end(&mut bytes)?;
                        protocol::verify_served(marker.as_bytes(), &bytes)?;
                        return confirm();
                    }
                    Err(paste::Error::NoMimeType | paste::Error::ClipboardEmpty) => {
                        // The serving thread may not have flushed its new offer yet.
                        // The startup watchdog bounds these native roundtrips.
                        thread::yield_now();
                    }
                    Err(error) => return Err(error.into()),
                }
            }
        })();
        if result.is_err() {
            let _ = protocol::write_status(io::stdout(), false);
            std::process::exit(1);
        }
    })?;
    prepared.serve()?;
    Ok(())
}

fn serve_x11(text: String, confirm: impl FnOnce() -> Result<()>) -> Result<()> {
    use x11rb::connection::Connection;
    use x11rb::protocol::{Event, xfixes, xproto::ConnectionExt};

    let clipboard = x11_clipboard::Clipboard::new()?;
    let getter = &clipboard.getter;
    let selection = getter.atoms.clipboard;
    xfixes::query_version(&getter.connection, 5, 0)?.reply()?;
    xfixes::select_selection_input(
        &getter.connection,
        getter.window,
        selection,
        xfixes::SelectionEventMask::SET_SELECTION_OWNER
            | xfixes::SelectionEventMask::SELECTION_CLIENT_CLOSE
            | xfixes::SelectionEventMask::SELECTION_WINDOW_DESTROY,
    )?
    .check()?;
    clipboard.store(selection, getter.atoms.utf8_string, text.as_bytes())?;
    let owner = clipboard.setter.window;
    let verify_owner = || -> Result<()> {
        ensure!(
            getter
                .connection
                .get_selection_owner(selection)?
                .reply()?
                .owner
                == owner,
            "Otro proceso reemplazó el portapapeles antes de confirmar la copia"
        );
        Ok(())
    };
    verify_owner()?;
    let served = clipboard.load(
        selection,
        getter.atoms.utf8_string,
        getter.atoms.property,
        SETUP_TIMEOUT,
    )?;
    protocol::verify_served(text.as_bytes(), &served)?;
    verify_owner()?;
    confirm()?;
    loop {
        match getter.connection.wait_for_event()? {
            Event::XfixesSelectionNotify(event) if event.owner != owner => return Ok(()),
            Event::Error(_) => bail!("Error del servidor X11"),
            _ => (),
        }
    }
}
