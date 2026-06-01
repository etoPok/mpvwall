use calloop::LoopSignal;
use libmpv2::events::Event;
use libmpv2::Mpv;
use tracing::{debug, error, warn};

use crate::bindings::mpv::mpv_error_string;

/// Convierte un error de mpv a su descripción textual usando mpv_error_string.
pub fn fmt_mpv_error(e: &libmpv2::Error) -> String {
    match e {
        libmpv2::Error::Raw(code) => {
            let s = unsafe {
                let ptr = mpv_error_string(*code);
                if ptr.is_null() {
                    format!("Raw({}) (unknown)", code)
                } else {
                    let cstr = std::ffi::CStr::from_ptr(ptr);
                    format!("Raw({}): {}", code, cstr.to_string_lossy())
                }
            };
            s
        }
        _ => format!("{}", e),
    }
}

pub fn process_mpv_events(mpv: &mut Mpv, loop_signal: &Option<LoopSignal>) {
    loop {
        match mpv.event_context_mut().wait_event(0.0) {
            Some(Ok(Event::EndFile(reason))) => {
                warn!("mpv: EndFile ({:?}), el loop debería reiniciar", reason);
            }
            Some(Ok(Event::Shutdown)) => {
                error!("mpv se cerró inesperadamente");
                if let Some(signal) = loop_signal {
                    signal.stop();
                }
                break;
            }
            Some(Ok(Event::LogMessage { text, .. })) => {
                debug!("mpv: {}", text.trim());
            }
            Some(Ok(_)) => {}
            Some(Err(e)) => {
                error!("Error en evento mpv: {}", fmt_mpv_error(&e));
                break;
            }
            None => break,
        }
    }
}
