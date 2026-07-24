use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from("../proto");
    let files = vec![
        "common.proto",
        "heartbeat.proto",
        "registration.proto",
        "mode.proto",
    ];
    let protos: Vec<PathBuf> = files.iter().map(|f| proto_dir.join(f)).collect();
    prost_build::compile_protos(&protos, &[&proto_dir])?;
    Ok(())
}
