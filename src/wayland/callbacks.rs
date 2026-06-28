use tracing::debug;
use wayland_client::{protocol::wl_callback::WlCallback, Connection, Dispatch, Proxy, QueueHandle};

use crate::app::state::App;

// WlCallback dispatch — just track the callback arrival.
// Rendering is driven by PTS timing in the event loop, not by wl_callback.
impl Dispatch<WlCallback, ()> for App {
    fn event(
        _state: &mut App,
        _proxy: &WlCallback,
        _event: <WlCallback as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<App>,
    ) {
        debug!("WlCallback fired");
        // The callback is just consumed. No rendering here —
        // rendering happens in the event loop based on PTS timing.
    }
}
