use anyhow::Result;
use libmpv2::Mpv;
use tracing::info;

pub fn init_mpv(gpu_api: &str) -> Result<Mpv> {
    let mpv = Mpv::with_initializer(|init| {
        init.set_property("terminal", "no")?;
        init.set_property("msg-level", "all=warn,vd=info")?;
        init.set_property("loop-file", "inf")?;
        init.set_property("loop", "inf")?;
        init.set_property("mute", true)?;
        init.set_property("audio", false)?;
        init.set_property("osc", false)?;
        init.set_property("osd-level", 0_i64)?;
        init.set_property("pause", false)?;
        init.set_property("hwdec", "auto-safe")?;
        init.set_property("keepaspect", false)?;
        init.set_property("input-default-bindings", false)?;
        init.set_property("input-vo-keyboard", false)?;
        init.set_property("input-cursor", false)?;
        init.set_property("vo", "libmpv")?;
        // Render API (vo=libmpv) solo soporta OpenGL. Ignorar el --gpu-api del usuario.
        init.set_property("gpu-api", "opengl")?;
        // NOTA: gpu-context NO se setea aquí; con vo=libmpv el contexto lo provee
        // la aplicación via mpv_render_context_create.
        // display-resample requiere timing del compositor que vo=libmpv no provee
        init.set_property("video-sync", "audio")?;
        // framedrop=vo es UB con vo=libmpv
        init.set_property("framedrop", "no")?;
        Ok(())
    })
    .map_err(|e| anyhow::anyhow!("Error inicializando libmpv: {}", e))?;

    info!(
        "libmpv inicializado (gpu-api={}, modo render API, sin ventana propia)",
        gpu_api
    );
    Ok(mpv)
}
