fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/invoker_manager.proto")?;
    Ok(())
}
