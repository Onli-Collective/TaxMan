use std::{
    collections::HashMap,
    future::ready,
    net::{Ipv4Addr, SocketAddrV4},
    sync::OnceLock,
};

use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use oauth2::TokenResponse;
use tokio::net::TcpListener;

const CLIENT_ID_VAR: &str = "TAXMAN_CLIENT_ID";
const CLIENT_SECRET_VAR: &str = "TAXMAN_CLIENT_SECRET";

/// Listens on the designated port for the next HTTP request,
/// and attempts to obtain an [AuthorizationCode](oauth2::AuthorizationCode) from the first request.
pub async fn listen_for_auth_code(
    port: u16,
    csrf_token: &str,
) -> Result<Option<oauth2::AuthorizationCode>, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)).await?;
    let (stream, _) = listener.accept().await?;
    let io = TokioIo::new(stream);

    let mut state = OnceLock::new();

    http1::Builder::new()
        .serve_connection(
            io,
            service_fn(|req: hyper::Request<hyper::body::Incoming>| {
                let query = req
                    .uri()
                    .query()
                    .map_or_else(
                        || Box::new(std::iter::empty()) as Box<dyn Iterator<Item = _>>,
                        |a| Box::new(a.split("&")),
                    )
                    .filter_map(|a| a.split_once("="))
                    .map(|(k, v)| (k.trim(), v.trim()))
                    .collect::<HashMap<_, _>>();
                let code = query
                    .get("code")
                    .zip(query.get("state"))
                    .and_then(|(&code, &state)| (state == csrf_token).then_some(code))
                    .map(ToString::to_string)
                    .map(oauth2::AuthorizationCode::new);

                if let Some(code) = code {
                    state.set(code).unwrap();
                }

                let mut res = hyper::Response::new(Full::<Bytes>::from("Done"));
                res.headers_mut().insert(
                    hyper::header::CONNECTION,
                    hyper::header::HeaderValue::from_static("close"),
                );

                ready(Result::<_, hyper::Error>::Ok(res))
            }),
        )
        .await?;

    Ok(state.take())
}

pub fn get_client_credentials_from_env() -> (oauth2::ClientId, oauth2::ClientSecret) {
    let client_id = oauth2::ClientId::new(
        std::env::var(CLIENT_ID_VAR).expect("could not get TAXMAN_CLIENT_ID environment variable"),
    );
    let secret = oauth2::ClientSecret::new(
        std::env::var(CLIENT_SECRET_VAR)
            .expect("could not get TAXMAN_CLIENT_SECRET environment variable"),
    );

    (client_id, secret)
}

pub async fn get_client_access_token() -> oauth2::AccessToken {
    let (client_id, secret) = get_client_credentials_from_env();
    let req = oauth2::basic::BasicClient::new(client_id)
        .set_client_secret(secret)
        .set_auth_type(oauth2::AuthType::RequestBody)
        .set_token_uri(taxman::auth::sandbox::TOKEN_URI.clone());

    let http = oauth2::reqwest::Client::builder().build().unwrap();
    req.exchange_client_credentials()
        .request_async(&http)
        .await
        .map(|a| a.access_token().clone())
        .expect("Could not authenticate as a client with HMRC api")
}

