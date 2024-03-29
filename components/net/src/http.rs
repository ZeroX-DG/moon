use reqwest::{Client, Method};

pub enum HttpResponse {
    Success(Vec<u8>),
    Failure(String),
}

static mut HTTP_CLIENT: Option<Client> = None;

fn get_http_client() -> &'static Client {
    unsafe {
        if HTTP_CLIENT.is_none() {
            HTTP_CLIENT = Some(Client::new());
        }
        HTTP_CLIENT.as_ref().expect("Unable to obtain http client")
    }
}

pub async fn request(method: &str, url: &str) -> HttpResponse {
    let client = get_http_client();
    let method = match method.to_lowercase().as_str() {
        "get" => Method::GET,
        _ => return HttpResponse::Failure(format!("Unsupported method: {}", method)),
    };

    let response = match client.request(method, url).send().await {
        Ok(response) => match response.bytes().await {
            Ok(bytes) => HttpResponse::Success(bytes.to_vec()),
            Err(e) => HttpResponse::Failure(e.to_string()),
        },
        Err(e) => HttpResponse::Failure(e.to_string()),
    };

    response
}
