use futures::future::{BoxFuture, FutureExt};
use hyper::{body::HttpBody as _, client::HttpConnector, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    print_structure(
        "ilias.php?ref_id=1836117&cmdClass=ilrepositorygui&cmdNode=yj&baseClass=ilrepositorygui"
            .to_string(),
        0,
    )
    .boxed()
    .await;
    Ok(())
}

async fn request_il_page(
    uri: String,
    client: &Client<HttpsConnector<HttpConnector>>,
) -> Result<Html, Box<dyn std::error::Error + Send + Sync>> {
    let req = Request::builder()
        .method(Method::GET)
        .uri("https://ilias.uni-freiburg.de/".to_owned() + &uri)
        .header("authority", "ilias.uni-freiburg.de")
        .header("upgrade-insecure-requests", 1)
        .header("dnt", 1)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.88 Safari/537.36")
        .header("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
        .header("sec-fetch-site", "same-origin")
        .header("sec-fetch-mode", "navigate")
        .header("sec-fetch-user", "?1")
        .header("sec-fetch-dest", "document")
        .header("referer", "https://ilias.uni-freiburg.de/ilias.php?baseClass=ilPersonalDesktopGUI&cmd=jumpToSelectedItems")
        .header("accept-language", "de-DE,de;q=0.9,en-US;q=0.8,en;q=0.7")
        .header("cookie", "iom_consent=00000000000000&1604408733754; ioam2018=000ef5c0cec6382585fa1559e:1632834334057:1604408734057:.uni-freiburg.de:6:ak025:dbs:noevent:1604871141009:nb8xt2; ilClientId=unifreiburg; _shibsession_64656661756c7468747470733a2f2f696c6961732e756e692d66726569627572672e64652f73686962626f6c657468=_0ed19f012091e81c7f66970654a4def0; PHPSESSID=qrb2h55lg6hh17cn9ckmnpiid0")
        .body(Body::empty()).unwrap();

    let mut resp = client.request(req).await?;
    let mut bytes = vec![];
    while let Some(chunk) = resp.body_mut().data().await {
        let chunk = chunk?;
        bytes.extend(&chunk[..]);
    }
    Ok(Html::parse_document(std::str::from_utf8(&bytes)?))
}


