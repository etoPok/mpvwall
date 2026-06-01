use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use anyhow::Context;
use calloop::{ping, timer::Timer, EventLoop};
use calloop_wayland_source::WaylandSource;
use tracing::{info, warn};
use wayland_client::{Connection, EventQueue};

use crate::app::state::App;
use crate::bindings::mpv::{mpv_render_context, mpv_render_context_set_update_callback};
use crate::mpv::callbacks::noop_update_callback;
use crate::mpv::events::process_mpv_events;
use crate::render::frame::render_frame;
use crate::runtime::signals::ctrlc_setup;

pub fn run(
    mut app: App,
    conn: Connection,
    queue: EventQueue<App>,
    ping_source: ping::PingSource,
    render_ctx: *mut mpv_render_context,
) -> anyhow::Result<()> {
    let mut event_loop: EventLoop<App> =
        EventLoop::try_new().context("Error creando event loop")?;

    let loop_signal = event_loop.get_signal();
    app.loop_signal = Some(loop_signal.clone());

    WaylandSource::new(conn.clone(), queue)
        .insert(event_loop.handle())
        .map_err(|e| anyhow::anyhow!("Error registrando fuente Wayland en event loop: {}", e))?;

    // Insertar PingSource: despierta el event loop cuando mpv llama al update callback.
    event_loop
        .handle()
        .insert_source(ping_source, |(), &mut (), _| {})
        .map_err(|e| anyhow::anyhow!("Error registrando PingSource en event loop: {}", e))?;

    // Timer para stats periódicas de rendimiento (cada 5 segundos).
    let stats_timer = Timer::from_duration(Duration::from_secs(5));
    event_loop
        .handle()
        .insert_source(stats_timer, |_, _, app| {
            if let Some(mpv) = &app.mpv {
                let frames = app.frame_count;
                let elapsed = app
                    .last_stats_time
                    .map(|t| t.elapsed().as_secs_f64())
                    .unwrap_or(5.0);
                let fps = if elapsed > 0.0 {
                    frames as f64 / elapsed
                } else {
                    0.0
                };

                // Consultar frame-drop del decoder.
                if let Ok(val) = mpv.get_property::<i64>("decoder-frame-drop-count") {
                    if val > 0 {
                        warn!(
                            "Stats: {:.1} fps, {} frames, decoder-drops: {}",
                            fps, frames, val
                        );
                    } else {
                        info!("Stats: {:.1} fps, {} frames, sin drops", fps, frames);
                    }
                } else {
                    info!("Stats: {:.1} fps, {} frames", fps, frames);
                }

                // Consultar fps estimado del video.
                if let Ok(val) = mpv.get_property::<f64>("estimated-vf-fps") {
                    info!("  estimated-vf-fps: {:.2}", val);
                }
            }
            app.frame_count = 0;
            app.last_stats_time = Some(Instant::now());
            // Re-programar el timer para las próximas stats.
            calloop::timer::TimeoutAction::ToDuration(Duration::from_secs(5))
        })
        .map_err(|e| anyhow::anyhow!("Error registrando stats timer: {}", e))?;

    app.last_stats_time = Some(Instant::now());

    info!("Event loop iniciado (sin polling). Ctrl+C para salir.");
    unsafe { ctrlc_setup(loop_signal) };

    // Se duerme indefinidamente hasta que mpv o Wayland despierten el loop.
    // No hay temporizador periódico — consumo de CPU ≈ 0 cuando no hay frames.
    event_loop
        .run(None, &mut app, |app| {
            if let Some(mpv) = &mut app.mpv {
                process_mpv_events(mpv, &app.loop_signal);
            }

            // Primer frame: solicitar frame callback ANTES de render para que
            // eglSwapBuffers commitee la surface incluyendo el frame request.
            if !app.first_render_attempted {
                app.first_render_attempted = true;
                if let Some(surface) = &app.surface {
                    if let Some(qh) = &app.qh {
                        app.wl_callback = Some(surface.frame(qh, ()));
                        app.frame_pending = true;
                    }
                }
                if let Some(rs) = &mut app.render_state {
                    if unsafe { render_frame(rs) } {
                        app.frame_count += 1;
                    }
                    app.first_frame_rendered = true;
                }
            }

            // Cuando mpv tiene datos nuevos (mpv_update_callback), solicitar
            // un frame de Wayland. El render REAL ocurre en Dispatch<WlCallback>
            // (vsync), donde render_frame llama mpv_render_context_update que
            // rearma el callback para el siguiente frame.
            let needs_render = app
                .mpv_update_state
                .map(|ptr| unsafe { (*ptr).needs_update.swap(false, Ordering::SeqCst) })
                .unwrap_or(false);

            if needs_render && !app.frame_pending {
                if let Some(surface) = &app.surface {
                    if let Some(qh) = &app.qh {
                        app.wl_callback = Some(surface.frame(qh, ()));
                        app.frame_pending = true;
                    }
                }
            }
        })
        .context("Error en event loop")?;

    // ------------------------------------------------------------------
    // Limpieza
    // ------------------------------------------------------------------

    info!("Saliendo limpiamente...");

    unsafe {
        mpv_render_context_set_update_callback(
            render_ctx,
            noop_update_callback,
            std::ptr::null_mut(),
        );
    }

    // Liberar el MpvUpdateState boxeado.
    if let Some(state_ptr) = app.mpv_update_state.take() {
        unsafe { drop(Box::from_raw(state_ptr)) };
    }

    if let Some(rs) = app.render_state.take() {
        drop(rs);
    }
    if let Some(mpv) = app.mpv.take() {
        drop(mpv);
    }

    if let Some(ls) = app.layer_surface.take() {
        ls.destroy();
    }
    if let Some(s) = app.surface.take() {
        s.destroy();
    }

    info!("Salida completa.");
    Ok(())
}
