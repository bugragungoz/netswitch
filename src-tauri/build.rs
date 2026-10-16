fn main() {
    // netSwitch reads traffic per app from Event Tracing for Windows and changes firewall rules;
    // both need administrator rights, so Windows asks for them when the app starts.
    let windows = tauri_build::WindowsAttributes::new().app_manifest(include_str!("app.manifest"));
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows)).expect("tauri build script");
}
