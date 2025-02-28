use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

use msfs::network::NetworkRequestBuilder;
use sentry::{ClientOptions, Transport, TransportFactory};

struct CustomSentryTransport {
    options: ClientOptions,
}

impl Transport for CustomSentryTransport {
    fn send_envelope(&self, envelope: sentry::Envelope) {
        // println!("[NAVIGRAPH]: Envelope request received {:?}", envelope);
        let mut body = Vec::new();
        envelope.to_writer(&mut body).unwrap();

        let dsn = self.options.dsn.as_ref().unwrap();
        let user_agent = self.options.user_agent.clone();
        let auth = dsn.to_auth(Some(&user_agent)).to_string();
        let url = dsn.envelope_api_url().to_string();

        let callback_received = Rc::new(RwLock::new(false));

        let cloned = Rc::clone(&callback_received);
        if let Some(res) = NetworkRequestBuilder::new(&url)
            .unwrap()
            .with_header(&format!("X-Sentry-Auth: {}", auth))
            .unwrap()
            .with_data(&mut body.clone())
            .with_callback(move |_e, _s| {
                let mut writer = cloned.write().unwrap();
                *writer = true;
                println!("[NAVIGRAPH]: Posted to Sentry");
            })
            .post(&String::from_utf8(body).unwrap())
        {
            let ec = res.error_code();

            println!("[NAVIGRAPH]: Res Code: {}", ec);
        } else {
            println!("[NAVIGRAPH]: Sentry failed to get res");
        };

        // while !(*callback_received.read().unwrap()) {}
    }
}

pub struct SentryTransportFactory;

impl TransportFactory for SentryTransportFactory {
    fn create_transport(&self, options: &sentry::ClientOptions) -> Arc<dyn sentry::Transport> {
        Arc::new(CustomSentryTransport {
            options: options.clone(),
        })
    }
}
