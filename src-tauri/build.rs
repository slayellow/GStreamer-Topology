fn main() {
    let windows = tauri_build::WindowsAttributes::new();
    let attributes = tauri_build::Attributes::new().windows_attributes(windows);

    // WindowsAttributes::new() embeds Tauri's Common Controls v6 manifest.
    // Without it, Windows can fail before main with TaskDialogIndirect errors.
    tauri_build::try_build(attributes).expect("failed to run Tauri build script");
}
