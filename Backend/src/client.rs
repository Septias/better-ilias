use hyper::Client;
use hyper_tls::HttpsConnector;
use std::sync::Arc;

type clientType = Arc<Client<HttpsConnector<HttpConnector>>>;
struct Client {
    client: clientType,
}
