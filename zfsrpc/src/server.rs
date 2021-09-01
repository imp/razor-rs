use tonic::{Code, Request, Response, Status};

use super::zfsrpc_proto::zfs_rpc_server::ZfsRpc;
use super::zfsrpc_proto::{BasicDatasetRequest, CreateFilesystemRequest, CreateVolumeRequest};
use super::zfsrpc_proto::{Empty, Filesystem, Volume};

pub mod service;

#[tonic::async_trait]
impl ZfsRpc for service::ZfsRpcService {
    async fn create_volume(
        &self,
        request: Request<CreateVolumeRequest>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();

        println!(
            "#########   create_volume() Got request: {:?}   #########",
            request
        );

        service::Volume::create(
            request.pool,
            request.name,
            Some(request.capacity),
            request.properties,
        )
        .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn create_filesystem(
        &self,
        request: Request<CreateFilesystemRequest>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();

        println!(
            "#########   create_filesystem() Got request: {:?}   #########",
            request
        );

        service::Filesystem::create(request.pool, request.name, request.properties)
            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn get_volume(
        &self,
        request: Request<BasicDatasetRequest>,
    ) -> Result<Response<Volume>, Status> {
        let request = request.into_inner();

        Ok(Response::new(
            service::Volume::get(request.pool, request.name)
                .map_err(|e| Status::new(Code::Internal, e.to_string()))?
                .into(),
        ))
    }

    async fn get_filesystem(
        &self,
        request: Request<BasicDatasetRequest>,
    ) -> Result<Response<Filesystem>, Status> {
        let request = request.into_inner();

        Ok(Response::new(
            service::Filesystem::get(request.pool, request.name)
                .map_err(|e| Status::new(Code::Internal, e.to_string()))?
                .into(),
        ))
    }

    async fn destroy_volume(
        &self,
        request: Request<BasicDatasetRequest>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();

        service::Volume::destroy(request.name)
            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn destroy_filesystem(
        &self,
        request: Request<BasicDatasetRequest>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();

        service::Filesystem::destroy(request.name)
            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

        Ok(Response::new(Empty {}))
    }
}