use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::warn;

use crate::domain::auth;

pub async fn auth_middleware(
    State(state): State<crate::api::handlers::AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match auth_header {
        Some(token) => match auth::verify_token(token, &state.jwt_secret) {
            Ok(claims) => {
                request.extensions_mut().insert(claims);
                next.run(request).await
            }
            Err(_) => {
                warn!("Invalid JWT token");
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"code": 401, "message": "认证已过期，请重新登录", "data": null})),
                )
                    .into_response()
            }
        },
        None => {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"code": 401, "message": "请先登录", "data": null})),
            )
                .into_response()
        }
    }
}
