use tracing::{info, warn};
use wayland_client::{
    delegate_noop,
    globals::GlobalListContents,
    protocol::{
        wl_compositor::WlCompositor,
        wl_output::{self, WlOutput},
        wl_registry::WlRegistry,
        wl_seat::WlSeat,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
};

use wayland_protocols::wp::viewporter::client::{
    wp_viewport::WpViewport, wp_viewporter::WpViewporter,
};

use crate::app::state::App;

impl Dispatch<WlRegistry, GlobalListContents> for App {
    fn event(
        _state: &mut App,
        _proxy: &WlRegistry,
        _event: <WlRegistry as Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<App>,
    ) {
    }
}

impl Dispatch<WlOutput, ()> for App {
    fn event(
        state: &mut App,
        _proxy: &WlOutput,
        event: wl_output::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<App>,
    ) {
        if let wl_output::Event::Mode { width, height, .. } = event {
            if state.width == 0 {
                info!("Output mode detected: {}x{}", width, height);
                state.width = width as u32;
                state.height = height as u32;
                state.output_width = width as u32;
                state.output_height = height as u32;
            }
        }
    }
}

delegate_noop!(App: ignore WlSurface);
delegate_noop!(App: ignore WlCompositor);
delegate_noop!(App: ignore WlSeat);
delegate_noop!(App: ignore ZwlrLayerShellV1);
delegate_noop!(App: ignore WpViewporter);
delegate_noop!(App: ignore WpViewport);

impl Dispatch<ZwlrLayerSurfaceV1, ()> for App {
    fn event(
        state: &mut App,
        proxy: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<App>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                info!("Configuration received: {}x{}", width, height);
                if !state.configured {
                    state.logical_width = width;
                    state.logical_height = height;

                    if let Some(vp) = &state.viewport {
                        vp.set_destination(width as i32, height as i32);
                        info!("Viewport destination set: {}x{}", width, height);
                    }

                    state.configured = true;
                    info!(
                        "Render target: {}x{} ( output: {}x{}, logical: {}x{} )",
                        state.width,
                        state.height,
                        state.output_width,
                        state.output_height,
                        state.logical_width,
                        state.logical_height
                    );
                }

                proxy.ack_configure(serial);
                if let Some(surface) = &state.surface {
                    surface.commit();
                }
            }
            zwlr_layer_surface_v1::Event::Closed => {
                warn!("Layer surface closed by the compositor");
                if let Some(signal) = &state.loop_signal {
                    signal.stop();
                }
            }
            _ => {}
        }
    }
}
