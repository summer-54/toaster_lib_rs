fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/invoker_manager.proto")?;
    tonic_prost_build::compile_protos("proto/testing_system.proto")?;
    Ok(())
}
