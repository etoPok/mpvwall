use std::os::raw::{c_char, c_int, c_void};

#[allow(non_camel_case_types)]
pub type mpv_handle = c_void;
#[allow(non_camel_case_types)]
pub type mpv_render_context = c_void;

pub const MPV_RENDER_API_TYPE_OPENGL: &[u8] = b"opengl\0";
pub const MPV_RENDER_PARAM_API_TYPE: c_int = 1;
pub const MPV_RENDER_PARAM_OPENGL_INIT_PARAMS: c_int = 2;
pub const MPV_RENDER_PARAM_OPENGL_FBO: c_int = 3;
pub const MPV_RENDER_PARAM_FLIP_Y: c_int = 4;
pub const MPV_RENDER_UPDATE_FRAME: u64 = 1;
pub const MPV_RENDER_PARAM_INVALID: c_int = 0;

#[repr(C)]
pub struct MpvOpenGLInitParams {
    pub get_proc_address: extern "C" fn(ctx: *mut c_void, name: *const c_char) -> *mut c_void,
    pub get_proc_address_ctx: *mut c_void,
}

#[repr(C)]
pub struct MpvRenderParam {
    pub type_: c_int,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct MpvOpenGLFbo {
    pub fbo: c_int,
    pub w: c_int,
    pub h: c_int,
    pub internal_format: c_int,
}

#[link(name = "mpv")]
extern "C" {
    pub fn mpv_render_context_create(
        res: *mut *mut mpv_render_context,
        mpv: *mut mpv_handle,
        params: *mut MpvRenderParam,
    ) -> c_int;
    pub fn mpv_render_context_render(
        ctx: *mut mpv_render_context,
        params: *mut MpvRenderParam,
    ) -> c_int;
    pub fn mpv_render_context_report_swap(ctx: *mut mpv_render_context);
    pub fn mpv_render_context_free(ctx: *mut mpv_render_context);
    pub fn mpv_render_context_set_update_callback(
        ctx: *mut mpv_render_context,
        callback: extern "C" fn(*mut c_void),
        callback_ctx: *mut c_void,
    );
    pub fn mpv_render_context_update(ctx: *mut mpv_render_context) -> u64;
    pub fn mpv_get_property(
        ctx: *mut mpv_handle,
        name: *const c_char,
        format: c_int,
        data: *mut c_void,
    ) -> c_int;
    pub fn mpv_error_string(error: c_int) -> *const c_char;
}

pub const MPV_FORMAT_STRING: c_int = 14;

#[repr(C)]
pub union MpvNodeData {
    pub string: *mut c_char,
}

#[repr(C)]
pub struct mpv_node {
    pub udata: MpvNodeData,
    pub format: c_int,
}
