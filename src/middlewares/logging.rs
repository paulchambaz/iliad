use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    Error, HttpMessage,
};

pub async fn log_request(
    req: ServiceRequest,
    next: Next<impl actix_web::body::MessageBody>,
) -> Result<ServiceResponse<impl actix_web::body::MessageBody>, Error> {
    let method = req.method().clone();
    let path = req.path().to_string();
    let peer = req.peer_addr().map(|a| a.ip().to_string()).unwrap_or_else(|| "-".to_string());

    let res = next.call(req).await?;

    let status = res.status().as_u16();
    let user = res.request().extensions().get::<String>().cloned().unwrap_or_else(|| "-".to_string());

    tracing::info!("{} {} {} {} {}", method, path, status, peer, user);

    Ok(res)
}
