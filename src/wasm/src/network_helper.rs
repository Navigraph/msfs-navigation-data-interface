use std::error::Error;

use msfs::network::{NetworkRequest, NetworkRequestBuilder, NetworkRequestState};

pub enum Method {
    Get,
}

pub struct NetworkHelper {
    request: NetworkRequest,
}

impl NetworkHelper {
    pub fn make_request(
        url: &str, method: Method, headers: Option<Vec<&str>>, data: Option<&mut [u8]>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut builder = NetworkRequestBuilder::new(url).ok_or("Failed to create NetworkRequestBuilder")?;

        // Add headers
        if let Some(headers) = headers {
            for header in headers {
                let new_builder = builder.with_header(header).ok_or("Failed to add header")?;
                builder = new_builder;
            }
        }

        // Add data
        if let Some(data) = data {
            let new_builder = builder.with_data(data);
            builder = new_builder;
        }

        // Send request
        let request = match method {
            Method::Get => builder.get().ok_or("Failed to send GET request")?,
        };

        Ok(Self { request })
    }

    pub fn response_state(&self) -> NetworkRequestState {
        self.request.state()
    }

    pub fn get_response(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        if self.request.state() != NetworkRequestState::DataReady {
            return Err("Request not finished yet".into());
        }

        let data = self.request.data().ok_or("Failed to get data")?;

        Ok(data)
    }
}
