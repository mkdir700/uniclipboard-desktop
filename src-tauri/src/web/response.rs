use serde::Serialize;
use warp::{Rejection, Reply};

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Serialize)]
pub struct ListResponse<T: Serialize> {
    list: Vec<T>,
    total: usize,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(code: u16, message: String, data: Option<T>) -> Self {
        Self {
            code,
            message,
            data,
        }
    }

    #[allow(dead_code)]
    pub fn success(data: T) -> Self {
        Self::new(200, "成功".to_string(), Some(data))
    }

    pub fn success_list(data: Vec<T>) -> ApiResponse<ListResponse<T>> {
        let total = data.len();
        let list_response = ListResponse { list: data, total };
        ApiResponse::new(200, "成功".to_string(), Some(list_response))
    }

    pub fn error(code: u16, message: String) -> Self {
        Self::new(code, message, None)
    }

    pub fn into_response(self) -> Result<impl Reply, Rejection> {
        Ok(warp::reply::with_status(
            warp::reply::json(&self),
            warp::http::StatusCode::from_u16(self.code).unwrap_or(warp::http::StatusCode::OK),
        ))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("测试数据".to_string());
        assert_eq!(response.code, 200);
        assert_eq!(response.message, "成功");
        assert_eq!(response.data, Some("测试数据".to_string()));
    }

    #[test]
    fn test_api_response_error() {
        let response = ApiResponse::<String>::error(400, "错误信息".to_string());
        assert_eq!(response.code, 400);
        assert_eq!(response.message, "错误信息");
        assert_eq!(response.data, None);
    }

    #[test]
    fn test_api_response_success_list() {
        let data = vec!["项目1".to_string(), "项目2".to_string(), "项目3".to_string()];
        let response = ApiResponse::success_list(data);
        assert_eq!(response.code, 200);
        assert_eq!(response.message, "成功");
        if let Some(list_response) = response.data {
            assert_eq!(list_response.total, 3);
            assert_eq!(list_response.list, vec!["项目1", "项目2", "项目3"]);
        } else {
            panic!("预期 data 字段不为 None");
        }
    }

    #[test]
    fn test_api_response_into_response() {
        let response = ApiResponse::success("测试数据".to_string());
        let result = response.into_response();
        assert!(result.is_ok());
        // 注意：这里我们无法直接测试 Reply 的内容，因为它是一个不透明的类型
        // 但我们至少可以确保它能够成功转换为 Response
    }
}
