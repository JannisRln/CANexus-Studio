use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(
        std::env::var_os("PBSLINTDOCK_ROOT")
            .expect("PBSLINTDOCK_ROOT must point to a checkout of pbtechlab/pbSlintDock"),
    );
    let library = root.join("ui/pbslintdock.slint");
    assert!(library.is_file(), "pbSlintDock UI not found: {}", library.display());
    let mut paths = HashMap::new();
    paths.insert("pbslintdock".into(), library);
    slint_build::compile_with_config(
        "ui/main.slint",
        slint_build::CompilerConfiguration::new().with_library_paths(paths),
    )
    .expect("could not compile CANexus docking spike");
    println!("cargo:rerun-if-changed=ui/main.slint");
}
