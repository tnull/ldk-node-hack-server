fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .protoc_arg("--proto_path=../src/proto")
        .compile(&["ldk_server_hack.proto"], &["*"])?;
    Ok(())
}