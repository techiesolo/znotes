#[cfg(target_os = "linux")]
fn set_webkitgtk_env_before_threads_start() {
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }
}

#[cfg(not(target_os = "linux"))]
fn set_webkitgtk_env_before_threads_start() {}

pub fn run() {
    set_webkitgtk_env_before_threads_start();
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running znotes");
}
