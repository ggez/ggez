//! Runs one of ggez's examples as an Android `NativeActivity`.

#![cfg(target_os = "android")]

include!(concat!(env!("OUT_DIR"), "/example.rs"));

#[no_mangle]
fn android_main(app: ggez::android::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("ggez"),
    );
    // stdout/stderr go nowhere on Android, so make sure panics end up in logcat.
    std::panic::set_hook(Box::new(|info| log::error!("{info}")));

    ggez::android::set_android_app(app);
    if let Err(e) = example::main() {
        log::error!("example exited with an error: {e:?}");
    }
}
