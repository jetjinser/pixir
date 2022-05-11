use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use hyper::{
    client::{connect::dns::Name, HttpConnector},
    service::Service,
};
use hyper_rustls::{ConfigBuilderExt, HttpsConnector, HttpsConnectorBuilder};
use rustls::ClientConfig;
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    error::ResolveError,
    lookup_ip::LookupIpIntoIter,
    TokioAsyncResolver,
};

const BP: [&'static str; 3] = ["app-api.pixiv.net", "www.pixiv.net", "app-api.pixiv.net"];

#[derive(Clone)]
pub struct PixivBypassResolver {
    inner: Arc<TokioAsyncResolver>,
}

pub struct SocketAddrs {
    iter: LookupIpIntoIter,
}

impl Iterator for SocketAddrs {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|ip_addr| SocketAddr::new(ip_addr, 0))
    }
}

impl PixivBypassResolver {
    pub fn new() -> Self {
        Self::default()
    }

    fn with_config_and_options(config: ResolverConfig, options: ResolverOpts) -> Self {
        let resolver = Arc::new(TokioAsyncResolver::tokio(config, options).unwrap());

        Self { inner: resolver }
    }

    pub fn into_http_connector(self) -> HttpConnector<Self> {
        HttpConnector::new_with_resolver(self)
    }

    pub fn into_https_connector(self) -> HttpsConnector<HttpConnector<Self>> {
        let mut http_connector = self.into_http_connector();
        http_connector.enforce_http(false);

        let mut config = ClientConfig::builder()
            .with_safe_defaults()
            .with_native_roots()
            .with_no_client_auth();

        config.enable_sni = false;

        let builder = HttpsConnectorBuilder::new()
            .with_tls_config(config)
            .https_or_http()
            .enable_http1();

        builder.wrap_connector(http_connector)
    }
}

impl Default for PixivBypassResolver {
    fn default() -> Self {
        Self::with_config_and_options(ResolverConfig::default(), ResolverOpts::default())
    }
}

impl Service<Name> for PixivBypassResolver {
    type Response = SocketAddrs;
    type Error = ResolveError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, name: Name) -> Self::Future {
        let resolver = self.inner.clone();

        dbg!(&name);

        let hostname = if BP.contains(&name.as_str()) {
            "www.pixivision.net".to_owned()
        } else {
            name.as_str().to_owned()
        };

        dbg!("[DNS] {} => {}", &name, &hostname);

        Box::pin(async move {
            let response = resolver.lookup_ip(hostname).await?;
            let addresses = response.into_iter();

            Ok(SocketAddrs { iter: addresses })
        })
    }
}
