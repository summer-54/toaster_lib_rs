mod auth;
mod judge;
mod master;

pub mod pb {
    tonic::include_proto!("invoker_manager");
}
