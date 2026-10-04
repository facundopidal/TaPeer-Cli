use std::io::{Read, Write};

use anyhow::{Context, Result, bail, ensure};

const REQUEST: &[u8; 4] = b"TXT1";
const READY: &[u8; 4] = b"RDY1";
const ERROR: &[u8; 4] = b"ERR1";
const ACCEPT: u8 = 1;

pub(super) fn write_request(mut output: impl Write, text: &str) -> Result<()> {
    output.write_all(REQUEST)?;
    output.write_all(&(text.len() as u64).to_le_bytes())?;
    output.write_all(text.as_bytes())?;
    output.flush()?;
    Ok(())
}

pub(super) fn read_request(mut input: impl Read) -> Result<String> {
    let mut magic = [0; 4];
    input.read_exact(&mut magic)?;
    ensure!(&magic == REQUEST, "Solicitud de portapapeles no válida");
    let mut length = [0; 8];
    input.read_exact(&mut length)?;
    let length = usize::try_from(u64::from_le_bytes(length))?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length)?;
    bytes.resize(length, 0);
    input.read_exact(&mut bytes)?;
    String::from_utf8(bytes).context("El portapapeles no contiene texto UTF-8 válido")
}

pub(super) fn write_status(mut output: impl Write, ready: bool) -> Result<()> {
    output.write_all(if ready { READY } else { ERROR })?;
    output.flush()?;
    Ok(())
}

pub(super) fn read_status(mut input: impl Read) -> Result<()> {
    let mut status = [0; 4];
    input
        .read_exact(&mut status)
        .context("El proceso de portapapeles terminó sin confirmar la copia")?;
    match &status {
        READY => Ok(()),
        ERROR => bail!("No se pudo preparar o verificar el portapapeles del sistema"),
        _ => bail!("Respuesta no válida del proceso de portapapeles"),
    }
}

pub(super) fn write_accept(mut output: impl Write) -> Result<()> {
    output.write_all(&[ACCEPT])?;
    output.flush()?;
    Ok(())
}

pub(super) fn read_accept(mut input: impl Read) -> Result<()> {
    let mut ack = [0];
    input.read_exact(&mut ack)?;
    ensure!(ack[0] == ACCEPT, "Confirmación de portapapeles no válida");
    Ok(())
}

pub(super) fn verify_served(expected: &[u8], actual: &[u8]) -> Result<()> {
    ensure!(
        expected == actual,
        "No se pudo verificar la copia del portapapeles"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_preserves_text_and_leaves_accept_unconsumed() {
        let text = "  café 🦀\n'quoted' \"double\"\r\n\t$(not executed)\n";
        let mut bytes = Vec::new();
        write_request(&mut bytes, text).unwrap();
        write_accept(&mut bytes).unwrap();
        let mut input = bytes.as_slice();
        assert_eq!(read_request(&mut input).unwrap(), text);
        read_accept(input).unwrap();
    }

    #[test]
    fn request_handles_empty_and_rejects_truncated_invalid_frames() {
        let mut bytes = Vec::new();
        write_request(&mut bytes, "").unwrap();
        assert_eq!(read_request(bytes.as_slice()).unwrap(), "");
        assert!(read_request(&bytes[..7]).is_err());
        bytes[0] = b'!';
        assert!(read_request(bytes.as_slice()).is_err());
        let mut bytes = b"TXT1".to_vec();
        bytes.extend_from_slice(&1u64.to_le_bytes());
        bytes.push(0xff);
        assert!(read_request(bytes.as_slice()).is_err());
    }

    #[test]
    fn only_explicit_complete_ready_status_succeeds() {
        for ready in [false, true] {
            let mut bytes = Vec::new();
            write_status(&mut bytes, ready).unwrap();
            assert_eq!(read_status(bytes.as_slice()).is_ok(), ready);
        }
        for status in [b"".as_slice(), b"RDY", b"????", b"ERR1"] {
            assert!(read_status(status).is_err());
        }
        assert!(read_accept([0].as_slice()).is_err());
        assert!(read_accept([].as_slice()).is_err());
    }

    #[test]
    fn closed_output_propagates_instead_of_confirming_success() {
        struct Closed;
        impl Write for Closed {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        assert!(write_request(Closed, "private").is_err());
        assert!(write_status(Closed, true).is_err());
        assert!(write_accept(Closed).is_err());
    }

    #[test]
    fn readiness_requires_exact_served_bytes_not_eof_or_an_old_probe() {
        let marker = b"application/x-tapeer-ready-current";
        assert!(verify_served(marker, marker).is_ok());
        for response in [
            b"".as_slice(),
            b"application/x-tapeer-ready-old",
            &marker[..10],
        ] {
            assert!(verify_served(marker, response).is_err());
        }
    }
}
