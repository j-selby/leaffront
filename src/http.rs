use std::sync::{mpsc, Arc};

use warp::Filter;

use tokio::runtime::{Builder, Runtime};

use std::net::SocketAddr;

pub enum RestAPIRequest {
    SetDay,
    SetNight,
    Reset
}

async fn start(http_endpoint: SocketAddr, sender: mpsc::Sender<RestAPIRequest>) {
    let sender_copy = Arc::new(sender);

    let reset_sender = sender_copy.clone();
    let reset = warp::path("api/reset").map(move || {
        reset_sender
            .send(RestAPIRequest::Reset)
            .expect("Failed to reset");
        "ok"
    });

    let day_sender = sender_copy.clone();
    let day = warp::path("api/reset").map(move || {
        day_sender
            .send(RestAPIRequest::SetDay)
            .expect("Failed to set night");
        "ok"
    });

    let night_sender = sender_copy;
    let night = warp::path("api/reset").map(move || {
        night_sender
            .send(RestAPIRequest::SetNight)
            .expect("Failed to set night");
        "ok"
    });

    let api = reset.or(day.or(night));
}

pub struct RestAPI {
    channel_receiver: mpsc::Receiver<RestAPIRequest>,
    _runtime: Arc<Runtime>,
}

impl RestAPI {
    pub fn read_message(&self) -> Option<RestAPIRequest> {
        match self.channel_receiver.try_recv() {
            Ok(value) => Some(value),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("RestAPI server terminated unexpectedly")
            }
        }
    }

    pub fn start(http_endpoint: &str) -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Failed to start Tokio threads");
        let runtime = Arc::new(runtime);

        let (request_tx, request_rx) = mpsc::channel();

        let api = RestAPI {
            channel_receiver: request_rx,
            _runtime: runtime.clone(),
        };

        let http_endpoint=  
                http_endpoint
                    .parse::<SocketAddr>()
                    .expect("Failed to parse socket address");

        runtime.spawn(async move {
            start(
                http_endpoint,
                request_tx,
            ).await;
        });

        api
    }
}
