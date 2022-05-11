mod pixiv_bypass_resolver;

use std::error::Error;

use hyper::{body::HttpBody, Body, Client, Request};
use pixiv_bypass_resolver::PixivBypassResolver;
use tokio::{fs::File, io::AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let connector = PixivBypassResolver::new().into_https_connector();

    let client: Client<_> = Client::builder().build(connector);

    let uri = "https://www.pixiv.net/artworks/98165208";

    let req = Request::get(uri)
        .header("Referer", "www.pixiv.net")
        .body(Body::empty())?;

    dbg!(&uri);

    let mut resp = client.request(req).await?;
    dbg!(&resp.status());
    dbg!(&resp.headers());

    let mut file = File::create("pix.png").await?;

    while let Some(chunk) = resp.body_mut().data().await {
        file.write_all(&chunk?).await?;
    }

    Ok(())
}
