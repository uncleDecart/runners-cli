use std::sync::Arc;
use tokio::sync::Mutex;

use std::time::Duration;

use dotenv::dotenv;
use runners_toolkit::fetchers::{self, establish_connection, Fetcher};
use runners_toolkit::gh_api::GitHubClient;
use std::env;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let token = env::var("GH_PAT").unwrap();
    let owner = env::var("OWNER").unwrap();
    // let clickhouse_url = env::var("CLICKHOUSE_URL").unwrap();
    // let clickhouse_user = env::var("CLICKHOUSE_USER").unwrap();
    // let clickhouse_password = env::var("CLICKHOUSE_PASSWORD").unwrap();
    // let clickhouse_db = env::var("CLICKHOUSE_DB").unwrap();

    let ghc = GitHubClient::new(token, owner);

    let client = establish_connection();
    // .with_option("async_insert", "1")
    // .with_option("wait_for_async_insert", "0");

    let items: Vec<(u64, Arc<Mutex<dyn Fetcher>>)> = vec![(
        60,
        Arc::new(Mutex::new(fetchers::RunnersSaturation::new(ghc, client))),
    )];

    for (interval_secs, device) in items {
        let device = Arc::clone(&device);
        tokio::spawn(async move {
            let interval = Duration::from_secs(interval_secs);
            loop {
                let res = device.lock().await.fetch().await;
                println!("fetched data {:?}", res);
                sleep(interval).await;
            }
        });
    }

    // Keep the main task alive forever
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
