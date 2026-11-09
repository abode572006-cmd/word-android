// Android library entrypoint for the native eframe host.
// This includes the upstream desktop application crate so its existing app, modules and helpers
// remain shared. The Android entrypoint creates a fresh document and launches the same WordApp UI.
#![allow(unsafe_code)]

include!("main.rs");

#[cfg(target_os = "android")]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    if let Err(error) = eframe::run_native(
        "WordCraft",
        options,
        Box::new(|cc| {
            let region = region::system();
            let doc = wordcraft_engine::blank_document(&region);
            let (file_dialog, dialogs) = file_dialogs::hook(&cc.egui_ctx);
            let mut session = Session::new(doc);
            session.system_region = region;
            let mut app = WordApp::new(session, services(file_dialog, &cc.egui_ctx));
            load_prefs(&mut app);
            wordcraft_engine::packs::use_font_store(true);
            app.start_screen();

            Ok(Box::new(App(
                app,
                None,
                None,
                dialogs,
                RawWindowState::default(),
            )))
        }),
    ) {
        log::error!("WordCraft Android event loop exited with an error: {error}");
    }
}
