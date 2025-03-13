use warp::Filter;

pub fn route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("download" / String)
        .and(warp::get())
        .map(|file_name: String| {
            // 这里应该实现文件下载逻辑
            // 现在只返回文件名作为示例
            warp::reply::html(format!("Downloading file: {}", file_name))
        })
}
