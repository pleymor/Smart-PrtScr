fn main() {
  // The application manifest (Common-Controls v6 dependency, same content as
  // tauri-build's default) is embedded by the linker instead of tauri-build so
  // that it also applies to test executables. Without it, a test binary that
  // links Tauri fails to start on Windows (STATUS_ENTRYPOINT_NOT_FOUND).
  let attributes = tauri_build::Attributes::new()
    .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
  tauri_build::try_build(attributes).expect("failed to run tauri-build");

  if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
    let manifest = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
      .join("windows-app-manifest.xml");
    println!("cargo:rerun-if-changed={}", manifest.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
  }
}
