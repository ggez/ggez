use crate::*;
use std::env;
use std::path;

#[test]
#[ignore]
pub fn context_build_tests() {
    let confs = vec![
        conf::Conf::default().window_mode(conf::WindowMode::default().dimensions(800.0, 600.0)),
        conf::Conf::default().window_mode(conf::WindowMode::default().dimensions(400.0, 400.0)),
        conf::Conf::default().window_mode(conf::WindowMode::default().resizable(false)),
        conf::Conf::default().window_mode(
            conf::WindowMode::default().fullscreen_type(conf::FullscreenType::Windowed),
        ),
        conf::Conf::default()
            .window_mode(conf::WindowMode::default().fullscreen_type(conf::FullscreenType::True)),
        conf::Conf::default().modules(conf::ModuleConf::default().audio(false)),
    ];
    for conf in confs.into_iter() {
        let mut cb = ContextBuilder::new("ggez_unit_tests", "ggez").conf(conf);
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            let mut path = path::PathBuf::from(manifest_dir);
            path.push("resources");
            cb = cb.add_resource_path(path);
        }
        let (c, _e) = cb.clone().build().unwrap();
        let (w, h) = graphics::drawable_size(&c);
        assert_eq!(w, cb.conf.window_mode.width.into());
        assert_eq!(h, cb.conf.window_mode.height.into());
        // Can't really test whether or not the window is resizable?
    }
}
