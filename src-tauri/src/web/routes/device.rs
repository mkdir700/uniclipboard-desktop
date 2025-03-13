use super::super::response::ApiResponse;
use crate::device::{get_device_manager, Device};
use crate::errors::LockError;
use warp::Filter;
impl warp::reject::Reject for LockError {}

pub fn route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("device").and(warp::get()).and_then(|| async {
        let device_manager = get_device_manager();
        let devices: Vec<Device> = device_manager
            .get_all_devices()
            .map_err(|e| warp::reject::custom(LockError(e.to_string())))?;
        ApiResponse::success_list(devices).into_response()
    })
}
