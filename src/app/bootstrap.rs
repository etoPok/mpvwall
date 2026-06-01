use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use calloop::ping;
use tracing::{info, warn};
use wayland_client::{globals::registry_queue_init, Connection};

use crate::bindings::mpv::{
    mpv_get_property, mpv_node, mpv_render_context, mpv_render_context_set_update_callback,
    MPV_FORMAT_STRING,
};
use crate::cli::args::Args;
use crate::mpv::callbacks::mpv_update_callback;
use crate::mpv::events::fmt_mpv_error;
use crate::mpv::init::init_mpv;
use crate::render::egl::init_egl;
use crate::render::mpv_render::create_render_context;
use crate::render::state::RenderState;
use crate::runtime::wakeup::MpvUpdateState;

use super::state::App;

pub struct BootstrapOutput {
    pub app: App,
    pub conn: Connection,
    pub queue: wayland_client::EventQueue<App>,
    pub ping_source: ping::PingSource,
    pub render_ctx: *mut mpv_render_context,
}

pub fn bootstrap(args: Args) -> Result<BootstrapOutput> {
    let gpu_api = args.gpu_api;
    let video_path_str = args.video_path;

    info!(
        "mpv-wallpaper iniciando con video: {} (gpu-api: {})",
        video_path_str, gpu_api
    );

    // Validar que el archivo de video existe antes de continuar
    let video_path = Path::new(&video_path_str);
    if !video_path.exists() {
        anyhow::bail!("El archivo de video no existe: {}", video_path.display());
    }
    let video_path = video_path
        .canonicalize()
        .context("Error resolviendo la ruta del video")?;
    let video_path_str = video_path.to_string_lossy().to_string();

    // ------------------------------------------------------------------
    // 1. Conectar a Wayland
    // ------------------------------------------------------------------

    let conn = Connection::connect_to_env()
        .context("No se pudo conectar al servidor Wayland. ¿Está WAYLAND_DISPLAY seteado?")?;

    // Obtener el puntero C al wl_display* para EGL.
    let wl_display_ptr = { conn.backend().display_ptr() as *mut c_void };

    let (globals, mut queue) =
        registry_queue_init::<App>(&conn).context("Error inicializando registry Wayland")?;
    let qh = queue.handle();

    let compositor = globals
        .bind(&qh, 4..=5, ())
        .context("El compositor no soporta wl_compositor")?;

    let layer_shell = globals
        .bind(&qh, 1..=4, ())
        .context("El compositor no soporta zwlr_layer_shell_v1")?;

    let output: Option<_> = globals.bind(&qh, 1..=4, ()).ok();
    if output.is_none() {
        warn!("No se detectó wl_output, el compositor asignará el monitor");
    }

    // ------------------------------------------------------------------
    // 2. Estado inicial
    // ------------------------------------------------------------------

    let mut app = App::new(compositor, layer_shell);
    app.output = output;
    app.qh = Some(qh.clone());

    queue
        .roundtrip(&mut app)
        .context("Error en roundtrip inicial")?;

    if app.width == 0 || app.height == 0 {
        warn!("Dimensiones del output no detectadas, usando 1920x1080 como fallback");
        app.width = 1920;
        app.height = 1080;
    }

    // ------------------------------------------------------------------
    // 3. Crear layer-shell surface
    // ------------------------------------------------------------------

    app.create_surfaces(&qh);

    let mut configure_attempts = 0;
    while !app.configured && configure_attempts < 50 {
        queue
            .blocking_dispatch(&mut app)
            .context("Error esperando configure")?;
        configure_attempts += 1;
    }

    if !app.configured {
        anyhow::bail!(
            "El compositor no envió configure tras {} intentos.",
            configure_attempts
        );
    }

    let wl_surface_ptr = app.wl_surface_ptr;
    if wl_surface_ptr.is_null() {
        anyhow::bail!("No se pudo obtener el puntero nativo de la wl_surface");
    }

    // ------------------------------------------------------------------
    // 4. Inicializar EGL/OpenGL
    // ------------------------------------------------------------------

    let width = app.width as i32;
    let height = app.height as i32;

    let (egl_display, egl_surface, egl_context, egl_window) = unsafe {
        init_egl(wl_display_ptr, wl_surface_ptr, width, height)
            .context("Error inicializando EGL")?
    };

    // ------------------------------------------------------------------
    // 5. Inicializar libmpv
    // ------------------------------------------------------------------

    let mpv = init_mpv(&gpu_api)?;

    // ------------------------------------------------------------------
    // 6. Crear mpv_render_context sobre el EGLContext activo
    // ------------------------------------------------------------------

    let render_ctx =
        unsafe { create_render_context(&mpv).context("Error creando mpv_render_context")? };

    // ------------------------------------------------------------------
    // 7. Configurar update callback de mpv + mecanismo de wakeup
    // ------------------------------------------------------------------

    let (ping, ping_source) = ping::make_ping().context("Error creando ping para wakeup")?;

    let update_state = Box::new(MpvUpdateState {
        needs_update: AtomicBool::new(false),
        ping,
    });
    let update_state_ptr = Box::into_raw(update_state);

    unsafe {
        mpv_render_context_set_update_callback(
            render_ctx,
            mpv_update_callback,
            update_state_ptr as *mut c_void,
        );
    }

    app.mpv_update_state = Some(update_state_ptr);

    // Guardar estado de renderizado y mpv en App.
    let rs = RenderState {
        egl_display,
        egl_surface,
        egl_context,
        egl_window,
        render_ctx,
        width,
        height,
    };
    app.render_state = Some(rs);
    app.mpv = Some(mpv);

    // ------------------------------------------------------------------
    // 8. Cargar video en mpv y esperar a que empiece la reproducción
    // ------------------------------------------------------------------

    app.mpv
        .as_mut()
        .unwrap()
        .command("loadfile", &[video_path_str.as_str(), "replace"])
        .map_err(|e| anyhow::anyhow!("Error cargando video en mpv: {}", e))?;

    info!("Video cargado, esperando reproducción...");

    // Esperar a que mpv cargue el archivo antes de verificar hwdec.
    let mut hwdec_checked = false;
    {
        let mpv_ref = app.mpv.as_mut().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match mpv_ref.event_context_mut().wait_event(0.5) {
                Some(Ok(libmpv2::events::Event::FileLoaded)) => {
                    info!("Video cargado por mpv, verificando aceleración por hardware...");
                    hwdec_checked = true;
                    break;
                }
                Some(Ok(libmpv2::events::Event::EndFile(reason))) => {
                    warn!("mpv: EndFile antes de cargar: {:?}", reason);
                    break;
                }
                Some(Ok(libmpv2::events::Event::Shutdown)) => {
                    anyhow::bail!("mpv se cerró inesperadamente durante carga");
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => {
                    warn!("Error en evento mpv durante carga: {}", fmt_mpv_error(&e));
                    break;
                }
                None => {}
            }
        }
    }

    // Verificar aceleración por hardware (ahora sí, después de FileLoaded).
    if hwdec_checked {
        unsafe {
            let prop = b"hwdec-current\0";
            let mut msg = std::mem::zeroed::<mpv_node>();
            let ret = mpv_get_property(
                app.mpv.as_ref().unwrap().ctx.as_ptr() as *mut crate::bindings::mpv::mpv_handle,
                prop.as_ptr() as *const c_char,
                MPV_FORMAT_STRING,
                &mut msg as *mut _ as *mut c_void,
            );
            if ret >= 0 {
                if !msg.udata.string.is_null() {
                    let hw = std::ffi::CStr::from_ptr(msg.udata.string).to_string_lossy();
                    info!("Aceleración por hardware activa: {}", hw);
                    libc::free(msg.udata.string as *mut c_void);
                } else {
                    warn!("hwdec-current: (null) — decodificación por CPU. Considera instalar VAAPI o usar --gpu-api vulkan");
                }
            } else {
                warn!(
                    "No se pudo consultar hwdec-current (código {}). Decodificación por CPU.",
                    ret
                );
            }
        }
    }

    info!("Iniciando render loop...");

    Ok(BootstrapOutput {
        app,
        conn,
        queue,
        ping_source,
        render_ctx,
    })
}
