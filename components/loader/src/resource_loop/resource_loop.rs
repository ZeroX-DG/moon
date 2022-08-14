use std::collections::{HashMap, VecDeque};

use super::{
    error::LoadError,
    request::LoadRequest,
};
use flume::{unbounded, Receiver, Sender, bounded, select};
use net::http::{self, HttpResponse};
use url::{Url, parser::URLParser};

pub struct ResourceLoop {
    request_channel: (Sender<LoadRequest>, Receiver<LoadRequest>),
}

impl ResourceLoop {
    pub fn new() -> Self {
        Self {
            request_channel: unbounded(),
        }
    }

    fn sender(&self) -> Sender<LoadRequest> {
        let (sender, _) = self.request_channel.clone();
        sender
    }

    pub fn start_loop(&self) -> Sender<LoadRequest> {
        let request_rx = self.request_channel.1.clone();
        let tick_rx = tick(1);

        let mut request_queue = VecDeque::new();
        let mut host_request_counter: HashMap<Url, u64> = HashMap::new();

        std::thread::spawn(move || loop {
            enum Event {
                Tick,
                QueueRequest(LoadRequest),
            }
            let event = select::Selector::new()
                .recv(&request_rx, |e| e.map(|req| Event::QueueRequest(req)))
                .recv(&tick_rx, |e| e.map(|_| Event::Tick))
                .wait();

            if event.is_err() {
                break;
            }

            match event.unwrap() {
                Event::QueueRequest(request) => {
                    request.listener().on_queued();
                    request_queue.push_back(request);
                },
                Event::Tick => {
                    if request_queue.is_empty() {
                        continue;
                    }
                    let request = request_queue.pop_front().unwrap();

                    let url = request.url();
                    let request_count = host_request_counter.get(&url).unwrap_or(&0) + 1;
                    host_request_counter.insert(url.clone(), request_count);

                    request.listener().on_started();
                    fetch_async(request);
                }
            }
        });
        self.sender()
    }
}

fn tick(ms: u64) -> Receiver<()> {
    let (tx, rx) = bounded(1);
    let _ = std::thread::spawn(move || loop {
        if tx.send(()).is_err() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(ms));
    });
    rx
}

/// Spawn a new thread & fetch the request. Return resource bytes (Vec<u8>) if success, otherwise a LoadError.
fn fetch_async(request: LoadRequest) {
    std::thread::spawn(|| fetch(request));
}

fn fetch(request: LoadRequest) {
    match request.url().scheme.as_str() {
        "file" => fetch_local(request),
        "http" | "https" => fetch_remote(request),
        "view-source" => fetch_source(request),
        scheme => request.listener().on_errored(LoadError::UnsupportedProtocol(scheme.to_string())),
    }
}

fn fetch_source(request: LoadRequest) {
    let target_url = URLParser::parse(&request.url().path.as_str(), None)
        .ok_or_else(|| LoadError::InvalidURL(request.url().as_str()));

    let listener = request.listener();
    match target_url {
        Ok(url) => fetch(LoadRequest::new(url, listener)),
        Err(error) => listener.on_errored(error),
    }
}

fn fetch_local(request: LoadRequest) {
    let fetch_result =
        std::fs::read(request.url().path.as_str()).map_err(|e| LoadError::IOError(e.to_string()));
    
    let listener = request.listener();
    match fetch_result {
        Ok(bytes) => listener.on_finished(bytes),
        Err(error) => listener.on_errored(error),
    }
}

fn fetch_remote(request: LoadRequest) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let fetch_result = match rt.block_on(http::request("GET", &request.url().as_str())) {
        HttpResponse::Success(bytes) => Ok(bytes),
        HttpResponse::Failure(err) => Err(LoadError::IOError(err)),
    };
    let listener = request.listener();
    match fetch_result {
        Ok(bytes) => listener.on_finished(bytes),
        Err(error) => listener.on_errored(error),
    }
}
