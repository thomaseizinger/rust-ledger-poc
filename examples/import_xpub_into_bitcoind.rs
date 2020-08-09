use jsonrpc_client::{Request, Response, SendRequest};
use serde::Deserialize;
use serde::Serialize;
use std::env::args;

fn main() -> anyhow::Result<()> {
    let mut args = args().skip(1);

    let auth = args.next().unwrap();
    let mut auth = auth.split(':').map(|s| s.to_owned());
    let wallet_name = args.next().unwrap();
    let external_desc = args.next().unwrap();
    let internal_desc = args.next().unwrap();

    let client = Client::new(
        "http://localhost:18443".parse()?,
        auth.next().unwrap(),
        auth.next().unwrap(),
    );

    let _ = client.createwallet(&wallet_name, true, true)?;

    let external_desc = client.getdescriptorinfo(external_desc)?.descriptor;
    let internal_desc = client.getdescriptorinfo(internal_desc)?.descriptor;

    let client = client.with_path(format!("/wallet/{}", wallet_name));

    let response = client.importmulti(
        vec![
            ImportMultiRequest {
                desc: external_desc,
                timestamp: 0,
                range: 0,
                internal: false,
                watchonly: true,
                keypool: true,
            },
            ImportMultiRequest {
                desc: internal_desc,
                timestamp: 0,
                range: 0,
                internal: true,
                watchonly: true,
                keypool: true,
            },
        ],
        ImportMultiOptions { rescan: true },
    )?;

    assert!(response[0].success);
    assert!(response[1].success);

    Ok(())
}

struct Client {
    inner: reqwest::blocking::Client,
    url: reqwest::Url,
    username: String,
    password: String,
}

impl Client {
    fn new(base_url: reqwest::Url, username: String, password: String) -> Self {
        Self {
            inner: Default::default(),
            url: base_url,
            username,
            password,
        }
    }

    fn with_path(self, path: String) -> Self {
        Self {
            url: self.url.join(&path).unwrap(),
            ..self
        }
    }
}

impl SendRequest for Client {
    type Error = reqwest::Error;

    fn send_request(&self, request: Request) -> Result<Response, Self::Error> {
        self.inner
            .post(self.url.clone())
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .json(&request)
            .send()?
            .json()
    }
}

impl BitcoinCore for Client {}

#[jsonrpc_client::api]
trait BitcoinCore {
    fn createwallet(
        &self,
        name: &str,
        disable_private_keys: bool,
        blank: bool,
    ) -> CreateWalletResponse;
    fn getdescriptorinfo(&self, descriptor: String) -> GetDescriptorInfoResponse;
    fn importmulti(
        &self,
        requests: Vec<ImportMultiRequest>,
        options: ImportMultiOptions,
    ) -> Vec<ImportMultiResult>;
}

#[derive(Serialize)]
struct ImportMultiRequest {
    desc: String,
    timestamp: u64,
    range: u64,
    internal: bool,
    watchonly: bool,
    keypool: bool,
}

#[derive(Serialize)]
struct ImportMultiOptions {
    rescan: bool,
}

#[derive(Deserialize)]
struct ImportMultiResult {
    success: bool
}

#[derive(Deserialize)]
struct CreateWalletResponse {
    #[serde(rename = "name")]
    _name: String,
    #[serde(rename = "warning")]
    _warning: String,
}

#[derive(Deserialize)]
struct GetDescriptorInfoResponse {
    descriptor: String,
}
