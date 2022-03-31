use std::os::unix::prelude::AsRawFd;
use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio_pipe::pipe;
use tonic::Status;

use super::*;

const ZFS: &str = "/usr/sbin/zfs";

pub async fn recv(mut input: tonic::Streaming<proto::SendSegment>) -> ZfsRpcResult<proto::Empty> {
    let origin: Option<String> = None;
    let response = Response::new(proto::Empty {});
    let segment = if let Some(segment) = input.message().await? {
        segment
    } else {
        return Ok(response);
    };

    let snapname = segment.name;
    let mut expected_sequence = segment.sequence + 1;
    debug!(sequence = segment.sequence, "Receiving message");

    let (reader, mut writer) = pipe()?;
    let fd = reader.as_raw_fd();
    task::spawn_blocking(move || max_pipe_size(fd))
        .await
        .map_err(join_to_status)??;

    let receiver = task::spawn_blocking(|| zfs::Zfs::receive(snapname, origin, false, reader));
    writer.write_all(&segment.buffer).await?;

    while let Some(segment) = input.message().await? {
        debug!(sequence = segment.sequence, "Receiving message");
        if expected_sequence == segment.sequence {
            expected_sequence = segment.sequence + 1;
        } else {
            let message = format!(
                "Message sequence mismatch: received {}, expected {}",
                segment.sequence, expected_sequence
            );
            return Err(Status::invalid_argument(message));
        }

        writer.write_all(&segment.buffer).await?;
    }

    receiver
        .await
        .map_err(join_to_status)?
        .map_err(zfs_to_status)?;

    Ok(response)
}

pub async fn recv_process(
    mut input: tonic::Streaming<proto::SendSegment>,
) -> ZfsRpcResult<proto::Empty> {
    let response = Response::new(proto::Empty {});
    let segment = if let Some(segment) = input.message().await? {
        segment
    } else {
        return Ok(response);
    };

    let snapname = segment.name;
    let mut expected_sequence = segment.sequence + 1;
    debug!(sequence = segment.sequence, "Receiving message");

    let mut recv = Command::new(ZFS);
    recv.arg("receive")
        .arg(&snapname)
        .stdin(Stdio::piped())
        .kill_on_drop(true);

    let mut receiver = recv.spawn()?;
    let mut stdin = receiver
        .stdin
        .take()
        .ok_or_else(|| tonic::Status::internal("Failed to get stdin from 'zfs receive'"))?;

    stdin.write_all(&segment.buffer).await?;

    while let Some(segment) = input.message().await? {
        debug!(sequence = segment.sequence, "Receiving message");
        if expected_sequence == segment.sequence {
            expected_sequence = segment.sequence + 1;
        } else {
            let message = format!(
                "Message sequence mismatch: received {}, expected {}",
                segment.sequence, expected_sequence
            );
            return Err(Status::invalid_argument(message));
        }

        stdin.write_all(&segment.buffer).await?;
    }

    let status = receiver.wait().await?;

    if !status.success() {
        if let Some(code) = status.code() {
            tracing::error!(code = code, "'zfs send` exit");
        } else {
            tracing::error!("'zfs send` killed by signal");
        }
    }

    Ok(response)
}
