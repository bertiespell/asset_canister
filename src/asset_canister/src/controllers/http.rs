use crate::database::chunks::get_chunk_by_order_id_for_file;
use crate::database::file::{get_file, FileID};
use crate::models::file::FileType;
use candid::{CandidType, Func, Nat};
use ic_cdk_macros::{self, query};
use num_traits::cast::ToPrimitive;
use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::borrow::Cow;
use std::str::FromStr;

const CACHE_HEADER_VALUE: &str = "public, max-age=100000000, immutable";

#[query]
fn http_request(request: HttpRequest) -> HttpResponse {
    match extract_route(&request.url) {
        Route::File(file_id) => start_streaming_file(file_id),
        _ => HttpResponse::not_found(),
    }
}

fn start_streaming_file(file_id: FileID) -> HttpResponse {
    if let Some(file) = get_file(&file_id) {
        let file_type = file.file_type.clone();
        let number_of_chunks = file.number_of_chunks.clone();
        if let Some(chunk) = get_chunk_by_order_id_for_file(&file, 0) {
            let streaming_strategy = if number_of_chunks > 1 {
                Some(StreamingStrategy::Callback {
                    callback: Func {
                        principal: ic_cdk::id(),
                        method: "http_request_streaming_callback".to_string(),
                    },
                    token: build_token(file_type, file_id, 1),
                })
            } else {
                None
            };

            let response = HttpResponse {
                status_code: 200,
                headers: vec![
                    HeaderField("Content-Type".to_string(), String::from(file_type.as_str())),
                    HeaderField("Cache-Control".to_string(), CACHE_HEADER_VALUE.to_string()),
                    HeaderField("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                ],
                body: Cow::Owned(chunk.chunk_data),
                streaming_strategy,
            };

            return response;
        }
    }

    HttpResponse::not_found()
}

#[query]
fn http_request_streaming_callback(token: Token) -> StreamingCallbackHttpResponse {
    continue_streaming_file(token)
}

fn continue_streaming_file(token: Token) -> StreamingCallbackHttpResponse {
    if let Route::File(file_id) = extract_route(&token.key) {
        let chunk_index = token.index.0.to_u64().unwrap();

        if let Some(file) = get_file(&file_id) {
            let file_type = file.file_type.clone();
            let number_of_chunks = file.number_of_chunks.clone();
            if let Some(chunk) = get_chunk_by_order_id_for_file(&file, chunk_index) {
                let token = if (chunk_index as u64) <= number_of_chunks {
                    Some(build_token(file_type, file_id, chunk_index + 1))
                } else {
                    None
                };
                return StreamingCallbackHttpResponse {
                    body: chunk.chunk_data,
                    token,
                };
            }
        }
    }

    StreamingCallbackHttpResponse {
        body: ByteBuf::new(),
        token: None,
    }
}

pub fn extract_route(path: &str) -> Route {
    let path = path
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_lowercase();

    if path.is_empty() {
        return Route::Other;
    }
    let parts: Vec<_> = path.split('/').collect();

    match parts[0] {
        "video" | "image" if parts.len() > 1 => {
            if let Ok(file_id) = FileID::from_str(parts[1]) {
                Route::File(file_id)
            } else {
                Route::Other
            }
        }

        _ => Route::Other,
    }
}

fn build_token(file_type: FileType, blob_id: u64, index: u64) -> Token {
    Token {
        key: format!("{}/{}", file_type.url_slug(), blob_id),
        content_encoding: String::default(),
        index: index.into(),
        sha256: None,
    }
}

pub enum Route {
    File(u64),
    Other,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct HeaderField(pub String, pub String);

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: ByteBuf,
}
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: Cow<'static, ByteBuf>,
    pub streaming_strategy: Option<StreamingStrategy>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Token {
    pub key: String,
    pub content_encoding: String,
    pub index: Nat,
    // The sha ensures that a client doesn't stream part of one version of an asset
    // followed by part of a different asset, even if not checking the certificate.
    pub sha256: Option<ByteBuf>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum StreamingStrategy {
    Callback { callback: Func, token: Token },
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StreamingCallbackHttpResponse {
    pub body: ByteBuf,
    pub token: Option<Token>,
}

impl HttpRequest {
    pub fn _header(&self, key: &str) -> Option<&String> {
        let key_lower = key.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == key_lower)
            .map(|(_, v)| v)
    }
}

impl HttpResponse {
    pub fn status_code(code: u16) -> HttpResponse {
        HttpResponse {
            status_code: code,
            headers: Vec::new(),
            body: Cow::default(),
            streaming_strategy: None,
        }
    }

    pub fn _gone() -> HttpResponse {
        HttpResponse::status_code(410)
    }

    pub fn not_found() -> HttpResponse {
        HttpResponse::status_code(404)
    }

    pub fn _moved_permanently(location: &str) -> HttpResponse {
        HttpResponse::_moved(301, location, None)
    }

    pub fn _moved_temporarily(location: &str, max_age: Option<u32>) -> HttpResponse {
        HttpResponse::_moved(302, location, max_age)
    }

    fn _moved(status_code: u16, location: &str, max_age: Option<u32>) -> HttpResponse {
        let mut headers = vec![HeaderField("Location".to_owned(), location.to_owned())];

        if let Some(max_age) = max_age {
            let value = format!("public, max-age={}", max_age);
            headers.push(HeaderField("Cache-Control".to_owned(), value));
        }

        HttpResponse {
            status_code,
            headers,
            body: Cow::default(),
            streaming_strategy: None,
        }
    }
}
