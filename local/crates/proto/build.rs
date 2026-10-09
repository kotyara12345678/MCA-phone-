use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proto");
    let canonical = root.canonicalize().expect("proto directory must exist");
    // canonicalize() on Windows returns a `\\?\` verbatim prefix that protoc
    // refuses to convert, so strip it and normalise to forward slashes before
    // the path ever reaches protoc (it does not map backslashes onto its
    // --proto_path prefixes).
    let root = canonical
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("//?/")
        .to_string();

    if std::env::var_os("PROTOC").is_none() {
        if let Ok(p) = protoc_bin_vendored::protoc_bin_path() {
            // SAFETY: set before the build script spawns protoc; no threads yet.
            unsafe { std::env::set_var("PROTOC", p) };
        }
    }

    let protos = [
        "mca/ai/v1/ai.proto",
        "mca/voice/v1/voice.proto",
        "grpc/health/v1/health.proto",
    ]
    .map(|f| format!("{root}/{f}"));
    let includes = [root.clone()];

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&protos, &includes)?;

    println!("cargo:rerun-if-changed={root}");
    Ok(())
}
