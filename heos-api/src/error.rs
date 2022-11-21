use crate::types::HeosErrorCode;

#[derive(thiserror::Error)]
pub enum HeosError {
    // TODO IO could be recovered!?
    // io, parse, whatever. Things that can't be dealt with anyhow ;)
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),

    #[error("Failed to invoke command '{command}'\n\teid: {eid}\n\ttext: {text} ")]
    InvalidCommand {
        command: String,
        eid: HeosErrorCode,
        text: String,
    },
    #[error("No HOES devices found in local network")]
    NoDeviceFound,
}
// We are still using a bespoke implementation of `Debug`
// to get a nice report using the error source chain
impl std::fmt::Debug for HeosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

fn error_chain_fmt(e: &impl std::error::Error, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
