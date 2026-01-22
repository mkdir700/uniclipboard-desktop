use tauri::http::{Request, StatusCode};
use uc_core::ids::RepresentationId;
use uc_core::BlobId;

/// Parsed UC protocol route.
/// 解析后的 UC 协议路由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UcRoute {
    Blob { blob_id: BlobId },
    Thumbnail { representation_id: RepresentationId },
}

/// Errors when parsing UC protocol requests.
/// 解析 UC 协议请求的错误。
#[derive(Debug, thiserror::Error)]
pub enum UcRequestError {
    #[error("Unsupported uc URI host")]
    UnsupportedHost,
    #[error("Missing resource id")]
    MissingId,
    #[error("Invalid resource id")]
    InvalidId,
}

impl UcRequestError {
    pub fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    pub fn response_message(&self) -> &'static str {
        match self {
            UcRequestError::UnsupportedHost => "Unsupported uc URI host",
            UcRequestError::MissingId => "Missing resource id",
            UcRequestError::InvalidId => "Invalid resource id",
        }
    }
}

/// Parse UC protocol request into route information.
/// 将 UC 协议请求解析为路由信息。
pub fn parse_uc_request(request: &Request<Vec<u8>>) -> Result<UcRoute, UcRequestError> {
    let uri = request.uri();
    let host = uri.host().unwrap_or_default();
    let path = uri.path();
    let resource_id = path.trim_start_matches('/');

    if resource_id.is_empty() {
        return Err(UcRequestError::MissingId);
    }

    if resource_id.contains('/') {
        return Err(UcRequestError::InvalidId);
    }

    match host {
        "blob" => Ok(UcRoute::Blob {
            blob_id: BlobId::from(resource_id),
        }),
        "thumbnail" => Ok(UcRoute::Thumbnail {
            representation_id: RepresentationId::from(resource_id),
        }),
        _ => Err(UcRequestError::UnsupportedHost),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_protocol_parsing() {
        let request = Request::builder()
            .uri("uc://thumbnail/rep-1")
            .body(Vec::new())
            .expect("build request");

        let route = parse_uc_request(&request).expect("expected uc request route");

        assert!(matches!(
            route,
            UcRoute::Thumbnail {
                representation_id,
            } if representation_id == RepresentationId::from("rep-1")
        ));
    }
}
