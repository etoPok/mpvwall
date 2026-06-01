use std::os::raw::c_void;

use wayland_client::Proxy;

/// Devuelve el *mut wl_proxy nativo de cualquier Proxy.
///
/// Requiere en Cargo.toml:
///   wayland-backend = { version = "0.3", features = ["client_system"] }
pub fn proxy_to_raw_ptr<P: Proxy>(proxy: &P) -> *mut c_void {
    // wayland_backend::ObjectId::as_ptr() devuelve el *mut wl_proxy nativo.
    // Esta es la API pública y estable del sys backend.
    proxy.id().as_ptr() as *mut c_void
}
