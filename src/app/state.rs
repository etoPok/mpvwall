use std::os::raw::c_void;
use std::ptr;
use std::time::Instant;

use calloop::LoopSignal;
use libmpv2::Mpv;
use wayland_client::protocol::{
    wl_callback::WlCallback,
    wl_compositor::WlCompositor,
    wl_output::WlOutput,
    wl_surface::WlSurface,
};
use wayland_client::QueueHandle;
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

use crate::render::state::RenderState;
use crate::runtime::wakeup::MpvUpdateState;

pub struct App {
    pub compositor: WlCompositor,
    pub layer_shell: ZwlrLayerShellV1,

    pub surface: Option<WlSurface>,
    pub layer_surface: Option<ZwlrLayerSurfaceV1>,

    pub width: u32,
    pub height: u32,

    pub output: Option<WlOutput>,
    pub loop_signal: Option<LoopSignal>,
    pub configured: bool,

    /// Puntero al wl_surface nativo (para wl_egl_window_create).
    /// Se obtiene a través de wayland_backend::sys.
    pub wl_surface_ptr: *mut c_void,

    /// Handle de la cola Wayland, necesario para solicitar frame callbacks.
    pub qh: Option<QueueHandle<App>>,

    /// Estado de renderizado EGL/mpv.
    pub render_state: Option<RenderState>,

    /// Instancia de mpv.
    pub mpv: Option<Mpv>,

    /// true cuando mpv tiene un frame nuevo listo para renderizar.
    /// Puntero raw al MpvUpdateState boxeado; se libera en la limpieza.
    pub mpv_update_state: Option<*mut MpvUpdateState>,

    /// true cuando se ha solicitado un wl_callback de frame y aún no ha disparado.
    pub frame_pending: bool,

    /// Callback de frame de Wayland. DEBE mantenerse vivo; si se hace drop,
    /// wayland-client envía wl_proxy_destroy y el compositor cancela el callback.
    pub wl_callback: Option<WlCallback>,

    /// Primer frame ya renderizado (para no renderizar dos veces).
    pub first_frame_rendered: bool,

    /// Primer intento de render ya hecho (para renderizar el primer frame sin depender de mpv_update_callback).
    pub first_render_attempted: bool,

    /// Contador de frames renderizados (para stats periódicas).
    pub frame_count: u64,

    /// Instante del último log de stats.
    pub last_stats_time: Option<Instant>,
}

impl App {
    pub fn new(compositor: WlCompositor, layer_shell: ZwlrLayerShellV1) -> Self {
        Self {
            compositor,
            layer_shell,
            surface: None,
            layer_surface: None,
            width: 0,
            height: 0,
            output: None,
            loop_signal: None,
            configured: false,
            wl_surface_ptr: ptr::null_mut(),
            qh: None,
            render_state: None,
            mpv: None,
            mpv_update_state: None,
            frame_pending: false,
            wl_callback: None,
            first_frame_rendered: false,
            first_render_attempted: false,
            frame_count: 0,
            last_stats_time: None,
        }
    }
}
