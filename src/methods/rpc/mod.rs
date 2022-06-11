use warp::{hyper::body::Bytes, Reply, http::StatusCode};

// Warp router
pub fn routes(_data: Bytes) -> impl Reply {
    warp::reply::with_status(Vec::new(), StatusCode::OK)
}
