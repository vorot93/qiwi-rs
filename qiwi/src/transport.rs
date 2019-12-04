use {
    headers::*,
    http::Method,
    log::*,
    reqwest_ext::*,
    serde::{Deserialize, Serialize},
    serde_json::Value,
    snafu::*,
    std::{
        collections::HashMap,
        fmt::{Debug, Display},
        future::Future,
        pin::Pin,
        sync::Arc,
    },
};

pub type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, Snafu)]
pub enum Error {
    NetworkError {
        source: StdError,
        backtrace: Backtrace,
    },
    ParseError {
        source: StdError,
        backtrace: Backtrace,
    },
}

impl Error {
    pub fn from_network_error<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        NetworkError.into_error(Box::new(error))
    }

    pub fn from_parse_error<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        ParseError.into_error(Box::new(error))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Rsp<T> {
    Error {
        #[serde(rename = "errorCode")]
        error: String,
    },
    OK(T),
}

pub trait Transport: Debug + Send + Sync + 'static {
    fn call(
        &self,
        endpoint: String,
        method: Method,
        params: &HashMap<&str, String>,
        body: Option<&Value>,
    ) -> Pin<Box<dyn Future<Output = Result<String, StdError>> + Send + 'static>>;
}

#[derive(Debug)]
pub struct RemoteCaller {
    pub http_client: reqwest::Client,
    pub addr: String,
    pub bearer: Option<String>,
}

impl Transport for RemoteCaller {
    fn call(
        &self,
        endpoint: String,
        method: Method,
        params: &HashMap<&str, String>,
        body: Option<&Value>,
    ) -> Pin<Box<dyn Future<Output = Result<String, StdError>> + Send + 'static>> {
        let client = self.http_client.clone();
        let uri = format!("{}/{}", self.addr, endpoint);
        trace!(
            "Sending request to endpoint {} with params: {:?}",
            endpoint,
            params
        );

        let mut req = client
            .request(method, &uri)
            .query(params)
            .typed_header(ContentType::json());
        if let Some(bearer) = self.bearer.as_ref() {
            req = req.bearer_auth(&bearer);
        }

        if let Some(body) = body {
            req = req.json(body);
        }

        Box::pin(async move {
            let rsp = req.send().await?;
            let err = rsp.error_for_status_ref().err();

            let data = rsp.text().await?;

            trace!("Received HTTP response: {}", data);

            if let Some(err) = err {
                return Err(format!("Received error {} with data: {}", err, data).into());
            }

            Ok(data)
        })
    }
}

#[derive(Clone, Debug)]
pub struct CallerWrapper {
    pub transport: Arc<dyn Transport>,
}

impl CallerWrapper {
    pub fn call<E, T>(
        &self,
        endpoint: E,
        method: Method,
        params: &HashMap<&str, String>,
        body: Option<&Value>,
    ) -> impl Future<Output = Result<Rsp<T>, Error>> + Send + 'static
    where
        E: Display,
        T: for<'de> Deserialize<'de> + Send + 'static,
    {
        let c = self
            .transport
            .call(endpoint.to_string(), method, params, body);
        async move {
            Ok(serde_json::from_str(&c.await.context(NetworkError)?)
                .map_err(Error::from_parse_error)?)
        }
    }
}
