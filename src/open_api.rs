use crate::{
    appv2::{EchoRequest, EchoResponse},
    models::{CreateKv, KvPair},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::appv2::health_handler,
        crate::appv2::echo_handler,
        crate::kv_axum::set_kv,
        crate::kv_axum::update_kv,
        crate::kv_axum::get_kv,
        crate::kv_axum::delete_kv
    ),
    components(schemas(EchoRequest, EchoResponse, CreateKv, KvPair)),
    info(
        title = "Combined Echo and Key-Value Store API",
        version = "1.0.0",
        description = "A combined API for echo and key-value store services"
    )
)]
pub struct ApiDoc;
