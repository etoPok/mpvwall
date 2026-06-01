//! mpv-wallpaper
//!
//! Renderiza un video como fondo de pantalla animado en Wayland/Hyprland.
//!
//! Arquitectura correcta (Wayland-native, sin hacks X11):
//!   1. Crea una wl_surface + zwlr_layer_surface_v1 (layer: BACKGROUND, fullscreen).
//!   2. Crea un wl_egl_window sobre la surface.
//!   3. Inicializa EGL: EGLDisplay → EGLContext → EGLSurface.
//!   4. Inicializa mpv_render_context (MPV_RENDER_API_TYPE_OPENGL) — mpv NO abre ninguna ventana.
//!   5. Loop: procesa eventos Wayland + mpv, renderiza frames con mpv_render_context_render,
//!      presenta con eglSwapBuffers.
//!
//! Uso:
//!   mpv-wallpaper /ruta/al/video.mp4

mod app;
mod bindings;
mod cli;
mod mpv;
mod render;
mod runtime;
mod wayland;

use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mpv_wallpaper=info".parse().unwrap()),
        )
        .init();

    let args = cli::args::parse();

    let output = app::bootstrap::bootstrap(args)?;

    runtime::event_loop::run(output.app, output.conn, output.queue, output.ping_source, output.render_ctx)
}
