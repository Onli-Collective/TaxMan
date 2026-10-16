pub mod util;

use std::marker::PhantomData;

use oauth2::{
    ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, TokenResponse,
    basic::BasicClient,
};

use taxman::prelude::*;

pub struct ReqwestClient {
    token: String,
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new(token: String) -> Self {
        Self {
            token,
            client: reqwest::Client::new(),
        }
    }
}

const fn as_method(method: taxman::prelude::Method) -> reqwest::Method {
    match method {
        Method::Get => reqwest::Method::GET,
        Method::Put => reqwest::Method::PUT,
        Method::Post => reqwest::Method::POST,
        Method::Delete => reqwest::Method::DELETE,
        Method::Options => reqwest::Method::OPTIONS,
        Method::Head => reqwest::Method::HEAD,
        Method::Patch => reqwest::Method::PATCH,
        Method::Trace => reqwest::Method::TRACE,
        Method::Query => reqwest::Method::QUERY,
    }
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    fn from(v: serde_json::Error) -> Self {
        Self::Serde(v)
    }
}

impl From<reqwest::Error> for Error {
    fn from(v: reqwest::Error) -> Self {
        Self::Reqwest(v)
    }
}

const HMRC_JSON_MIME: &str = "application/vnd.hmrc.2.0+json";

impl Client for ReqwestClient {
    type Res<T> = Result<T, Error>;

    async fn request<E, O: Operation>(
        &self,
        body: O,
        when: <O::Path as Path>::PathParams,
        env: E,
    ) -> Self::Res<O::Response>
    where
        Status<O>: for<'de> DeserializeSeed<'de, Value = O::Response>,
        <O::Path as Path>::Api: Server<E>,
        E: Environment<O::Scenario>,
    {
        let request = self.client.request(
            as_method(O::METHOD),
            // TODO: replace this with a nicer [Url::join]
            format!(
                "{base}{path}",
                base = <<O::Path as Path>::Api as Server<E>>::URL,
                path = O::Path::url(when)
            ),
        );

        // Add the body for applicable HTTP methods.
        let request = if matches!(
            as_method(O::METHOD),
            reqwest::Method::GET | reqwest::Method::DELETE | reqwest::Method::HEAD
        ) {
            request
        } else {
            request.body(serde_json::to_string(&body)?)
        };

        let request = request
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", HMRC_JSON_MIME);

        // TODO: Add fraud prevention headers...

        // Apply any Gov-Test-Scenario header (if present/applicable).
        let request = env.apply::<Self>(request);

        // Send the request...
        let res = request.send().await?;
        let status = res.status().as_u16();
        let json = res.text().await?;

        // Deserialize into the target type, using the HTTP status information as a hint to the deserialize impl.
        let mut deserializer = serde_json::Deserializer::from_str(&json);
        Status {
            status,
            operation: PhantomData::<O>,
        }
        .deserialize(&mut deserializer)
        .map_err(Error::from)
    }

    type PendingRequest<'r> = reqwest::RequestBuilder;
    fn apply_header<'r>(
        request: Self::PendingRequest<'r>,
        header: &'static str,
        value: &'static str,
    ) -> Self::PendingRequest<'r> {
        request.header(header, value)
    }
}

const LISTENING_PORT: u16 = 5173;

#[tokio::test]
async fn test_client() -> Result<(), Box<dyn std::error::Error>> {
    let client_id = ClientId::new(std::env::var("CLIENT_ID")?);
    let client_secret = ClientSecret::new(std::env::var("CLIENT_SECRET")?);
    let oauth = oauth2::basic::BasicClient::new(client_id)
        .set_auth_type(oauth2::AuthType::RequestBody)
        .set_client_secret(client_secret)
        .set_auth_uri(taxman::auth::sandbox::AUTH_URI.clone())
        .set_token_uri(taxman::auth::sandbox::TOKEN_URI.clone())
        .set_redirect_uri(RedirectUrl::new(format!(
            "http://localhost:{LISTENING_PORT}"
        ))?);

    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = oauth
        .authorize_url(CsrfToken::new_random)
        .add_scopes([
            taxman::auth::scope::self_assessment::WRITE.clone(),
            taxman::auth::scope::self_assessment::READ.clone(),
        ])
        .set_pkce_challenge(challenge)
        .url();

    println!("Please navigate to:\n{auth_url}");

    let code = util::listen_for_auth_code(LISTENING_PORT, csrf_token.secret().as_str())
        .await?
        .expect("Did not get the code!");

    let client = oauth2::reqwest::Client::builder()
        // .redirect(oauth2::reqwest::redirect::Policy::none())
        .build()
        .expect("client should build");

    // TODO: this will error out without providing the client secret because HMRC does not support
    //       secret-less OAuth stuff...
    let token = oauth
        .exchange_code(code)
        .set_pkce_verifier(verifier)
        .request_async(&client)
        .await
        .expect("did not get access token");

    let client = ReqwestClient::new(token.access_token().secret().clone());

    {
        use taxman::test_impl::list_all_businesses::*;
        let res: Result<Result<Response, Error>, crate::Error> = client
            .request(
                ListAllBusinesses,
                Params {
                    nino: "ON289642C".to_string(),
                },
                Sandbox(None),
            )
            .await;

        println!("{res:?}")
    }

    Ok(())
}
