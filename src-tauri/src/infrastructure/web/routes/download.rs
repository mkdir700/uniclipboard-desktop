use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use warp::http::StatusCode;
use warp::hyper::Body;
use warp::{Filter, Rejection, Reply};
use anyhow::Result;
use log::{error, info};

use crate::infrastructure::storage::db::models::clipboard_record::DbClipboardRecord;
use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::storage::record_manager::ClipboardRecordManager;

/// 下载路由
pub fn route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let record_manager = Arc::new(ClipboardRecordManager::new(100));
    let file_storage =
        Arc::new(FileStorageManager::new().expect("Failed to create FileStorageManager"));

    warp::path!("download" / String)
        .and(warp::get())
        .and(with_record_manager(record_manager))
        .and(with_file_storage(file_storage))
        .and_then(handle_download)
}

/// 注入记录管理器
fn with_record_manager(
    record_manager: Arc<ClipboardRecordManager>,
) -> impl Filter<Extract = (Arc<ClipboardRecordManager>,), Error = Infallible> + Clone {
    warp::any().map(move || record_manager.clone())
}

/// 注入文件存储管理器
fn with_file_storage(
    file_storage: Arc<FileStorageManager>,
) -> impl Filter<Extract = (Arc<FileStorageManager>,), Error = Infallible> + Clone {
    warp::any().map(move || file_storage.clone())
}

/// 处理下载请求
async fn handle_download(
    record_id: String,
    record_manager: Arc<ClipboardRecordManager>,
    file_storage: Arc<FileStorageManager>,
) -> Result<impl Reply, Rejection> {
    info!("Download request for record: {}", record_id);
    let record = record_manager
        .get_record_by_id(&record_id)
        .await
        .map_err(|e| {
            error!("Failed to get record: {:?}", e);
            warp::reject::not_found()
        })?
        .ok_or_else(|| {
            error!("Record not found: {}", record_id);
            warp::reject::not_found()
        })?;

    let file_path = record.local_file_path.as_ref().ok_or_else(|| {
        error!("Invalid file path for record: {}", record_id);
        warp::reject::not_found()
    })?;

    let (stream, file_size) = file_storage
        .create_stream(&PathBuf::from(&file_path))
        .await
        .map_err(|_| warp::reject::not_found())?;
    let body = Body::wrap_stream(stream);

    // 设置响应头
    let mut response = warp::http::Response::builder()
        .status(StatusCode::OK);
    
    // 添加头部
    let content_type = get_content_type(&record)
        .map_err(|_| warp::reject::not_found())?;
    response = response.header("Content-Type", content_type);
    
    let filename = get_filename(file_path)
        .map_err(|_| warp::reject::not_found())?;
    response = response.header("Content-Disposition", 
        format!("attachment; filename=\"{}\"", filename));
    
    response = response.header("Content-Length", file_size.to_string());
    response = response.header("Accept-Ranges", "bytes");
    
    let response = response.body(body)
        .map_err(|_| warp::reject::not_found())?;

    Ok(response)
}

/// 根据记录获取内容类型
fn get_content_type(record: &DbClipboardRecord) -> Result<String, warp::Rejection> {
    match record.content_type.as_str() {
        "text" => Ok("text/plain; charset=utf-8".to_string()),
        "image" => {
            let file_path = record.local_file_path.as_ref().ok_or_else(|| {
                error!("Invalid file path for image record: {}", record.id);
                warp::reject::not_found()
            })?;

            // 从文件路径中提取图片格式
            let path = PathBuf::from(&file_path);
            let content_type = match path.extension().and_then(|ext| ext.to_str()) {
                Some("png") => "image/png".to_string(),
                Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
                Some("gif") => "image/gif".to_string(),
                Some("webp") => "image/webp".to_string(),
                _ => "application/octet-stream".to_string(),
            };
            Ok(content_type)
        }
        "link" => Ok("text/plain; charset=utf-8".to_string()),
        "code_snippet" => Ok("text/plain; charset=utf-8".to_string()),
        "rich_text" => Ok("text/html; charset=utf-8".to_string()),
        _ => Ok("application/octet-stream".to_string()),
    }
}

/// 根据记录获取文件名
fn get_filename(file_path: &str) -> Result<String, warp::Rejection> {
    let path = PathBuf::from(&file_path);
    path.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            error!("Invalid file path: cannot extract filename");
            warp::reject::not_found()
        })
        .map(|name| name.to_string())
}
