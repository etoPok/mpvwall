use std::os::raw::{c_int, c_void};
use std::ptr;

use crate::bindings::mpv::{
    mpv_render_context_render, mpv_render_context_report_swap, mpv_render_context_update,
    MpvOpenGLFbo, MpvRenderParam, MPV_RENDER_PARAM_FLIP_Y, MPV_RENDER_PARAM_INVALID,
    MPV_RENDER_PARAM_OPENGL_FBO, MPV_RENDER_UPDATE_FRAME,
};
use crate::bindings::egl::eglSwapBuffers;
use crate::render::state::RenderState;

/// Devuelve true si se renderizó un frame, false si no había frame nuevo.
pub unsafe fn render_frame(rs: &mut RenderState) -> bool {
    let flags = mpv_render_context_update(rs.render_ctx);
    let has_frame = flags & MPV_RENDER_UPDATE_FRAME != 0;

    if has_frame {
        let mut fbo = MpvOpenGLFbo {
            fbo: 0,
            w: rs.width,
            h: rs.height,
            internal_format: 0,
        };
        let mut flip_y: c_int = 1;

        let mut params = [
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_OPENGL_FBO,
                data: &mut fbo as *mut _ as *mut c_void,
            },
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_FLIP_Y,
                data: &mut flip_y as *mut _ as *mut c_void,
            },
            MpvRenderParam {
                type_: MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        mpv_render_context_render(rs.render_ctx, params.as_mut_ptr());
        eglSwapBuffers(rs.egl_display, rs.egl_surface);
        mpv_render_context_report_swap(rs.render_ctx);
    } else {
        // Swap sin render para commitar la surface Wayland.
        // Sin commit el wl_callback.frame() nunca se procesa y el
        // loop de frame callbacks muere.
        eglSwapBuffers(rs.egl_display, rs.egl_surface);
    }
    has_frame
}
