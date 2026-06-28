use calloop::ping;

pub struct Notifier(pub ping::Ping);

unsafe impl Send for Notifier {}
