use std::sync::atomic::AtomicBool;

use calloop::ping;

/// Estado compartido entre el callback de mpv (hilo de decodificación)
/// y el event loop principal.
pub struct MpvUpdateState {
    pub needs_update: AtomicBool,
    pub ping: ping::Ping,
}
