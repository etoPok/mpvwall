use std::sync::atomic::{AtomicUsize, Ordering};

use tracing::debug;
use wayland_client::{
    protocol::wl_callback::WlCallback,
    Connection, Dispatch, Proxy, QueueHandle,
};

use crate::app::state::App;
use crate::render::frame::render_frame;

pub static WL_CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);

impl Dispatch<WlCallback, ()> for App {
    fn event(
        state: &mut App,
        _proxy: &WlCallback,
        _event: <WlCallback as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<App>,
    ) {
        let wl_callback_count = WL_CALLBACK_COUNT.fetch_add(1, Ordering::SeqCst);
        debug!("WlCallback::event llamado {} veces", wl_callback_count + 1);

        state.frame_pending = false;

        // Solicitar el siguiente frame callback ANTES de render, para que
        // eglSwapBuffers (dentro de render_frame) commitee la surface
        // incluyendo esta solicitud. Sin commit, el compositor ignora el
        // wl_callback y el loop de frames muere.
        if let Some(surface) = &state.surface {
            if let Some(qh) = &state.qh {
                state.wl_callback = Some(surface.frame(qh, ()));
                state.frame_pending = true;
            }
        }

        // render_frame SIEMPRE llama eglSwapBuffers (con o sin frame),
        // lo que commitea la surface y entrega el frame request al servidor.
        // Internamente también llama mpv_render_context_update() que rearma
        // mpv_update_callback para el próximo frame.
        if let Some(rs) = &mut state.render_state {
            if unsafe { render_frame(rs) } {
                state.frame_count += 1;
            }
        }
    }
}
