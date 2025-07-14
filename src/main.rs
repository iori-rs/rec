mod config;

use std::str::FromStr;

use clap::Parser;
use config::Config;
use iori::{
    HttpClient,
    cache::{
        IoriCache,
        opendal::{Configurator, Operator},
    },
    download::ParallelDownloader,
    hls::HlsLiveSource,
    merge::IoriMerger,
    reqwest::{
        Client,
        header::{HeaderMap, HeaderName, HeaderValue},
    },
};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    key: Option<String>,

    #[clap(short = 'H', long = "header")]
    headers: Vec<String>,

    m3u8_url: String,
    prefix: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .try_from_env()
                .unwrap_or_else(|_| "info,iori::hls=warn,iori::download=warn".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    let config = Config::load()?;
    let args = Args::parse();

    let operator = Operator::new(config.s3.into_builder())?.finish();

    let mut headers = HeaderMap::new();
    for header in &args.headers {
        let (key, value) = header.split_once(':').expect("Invalid header");
        headers.insert(
            HeaderName::from_str(key).expect("Invalid header name"),
            HeaderValue::from_str(value).expect("Invalid header value"),
        );
    }

    let client = HttpClient::new(Client::builder().default_headers(headers));
    let source = HlsLiveSource::new(client, args.m3u8_url, args.key.as_deref(), None);

    let cache = IoriCache::opendal(
        operator.clone(),
        &args.prefix,
        false,
        Some("application/octet-stream".to_string()),
    );
    let merger = IoriMerger::skip();

    ParallelDownloader::builder()
        .cache(cache)
        .merger(merger)
        .download(source)
        .await?;

    Ok(())
}
