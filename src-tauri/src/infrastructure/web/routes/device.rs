use crate::{application::device_service::get_device_manager, domain::device::Device, utils::errors::LockError};

use super::super::response::ApiResponse;
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
