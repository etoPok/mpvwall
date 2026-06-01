use std::os::raw::{c_int, c_void};

#[link(name = "wayland-egl")]
extern "C" {
    pub fn wl_egl_window_create(surface: *mut c_void, width: c_int, height: c_int) -> *mut c_void;
    pub fn wl_egl_window_destroy(egl_window: *mut c_void);
}
