use anyhow::{anyhow, Result};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use msfs::network::{NetworkRequest, NetworkRequestState};

struct NetworkRequestExecutor {
    request: NetworkRequest,
}

impl NetworkRequestExecutor {
    pub fn new(request: NetworkRequest) -> Self {
        Self { request }
    }
}

impl Future for NetworkRequestExecutor {
    type Output = Result<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let status = self.request.state();
        let error_code = self.request.error_code();

        match status {
            NetworkRequestState::New => Poll::Pending,
            NetworkRequestState::WaitingForData => Poll::Pending,
            NetworkRequestState::DataReady => {
                let res = self.request.data().ok_or(anyhow!("No data with response"));

                Poll::Ready(res)
            }
            NetworkRequestState::Invalid => Poll::Ready(Err(anyhow!(
                "Network request invalid with error code {error_code}",
            ))),
            NetworkRequestState::Failed => Poll::Ready(Err(anyhow!(
                "Network request failed with error code {error_code}"
            ))),
        }
    }
}

pub trait AsyncNetworkRequest {
    async fn wait_for_data(&self) -> Result<Vec<u8>>;
}

impl AsyncNetworkRequest for NetworkRequest {
    async fn wait_for_data(&self) -> Result<Vec<u8>> {
        NetworkRequestExecutor::new(*self).await
    }
}
