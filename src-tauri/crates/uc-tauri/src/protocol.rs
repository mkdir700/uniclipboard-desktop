use tauri::http::{Request, StatusCode};
use uc_core::ids::RepresentationId;
use uc_core::BlobId;

/// # Behavior / 行为
///
/// Parsed UC protocol route.
/// 解析后的 UC 协议路由。
///
/// Represents the parsed resource type from a UC protocol URI (`uc://host/id`).
/// 表示从 UC 协议 URI (`uc://host/id`) 解析出的资源类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UcRoute {
    Blob { blob_id: BlobId },
    Thumbnail { representation_id: RepresentationId },
}

/// # Behavior / 行为
///
/// Errors when parsing UC protocol requests.
/// 解析 UC 协议请求的错误。
///
/// Represents validation failures during UC protocol URI parsing.
/// 表示 UC 协议 URI 解析过程中的验证失败。
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

/// # Behavior / 行为
///
/// Parse UC protocol request into route information.
/// 将 UC 协议请求解析为路由信息。
///
/// Extracts the host and resource ID from the URI and validates the format.
/// 从 URI 中提取主机和资源 ID 并验证格式。
///
/// # Examples / 示例
///
/// ```no_run
/// use tauri::http::Request;
/// use uc_tauri::protocol::parse_uc_request;
///
/// let request = Request::builder()
///     .uri("uc://blob/blob-123")
///     .body(Vec::new())
///     .unwrap();
///
/// let route = parse_uc_request(&request)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
