use anyhow::Result;
use dotenv::dotenv;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
struct RunnerList {
    runners: Vec<Runner>,
}

#[derive(Debug, Deserialize)]
struct Runner {
    id: u64,
    name: String,
    busy: bool,
}

fn main() -> Result<()> {
    dotenv().ok();

    let token = env::var("GH_PAT")?;
    let owner = env::var("OWNER")?;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] != "show" {
        eprintln!("Usage: cargo run -- show");
        return Ok(());
    }

    let client = Client::new();
    let url = format!("https://api.github.com/orgs/{}/actions/runners", owner);

    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("token {}", token))
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, "github-org-runner-cli")
        .send()?
        .error_for_status()?;

    let runners: RunnerList = res.json()?;

    println!(
        "👀 GitHub Self-hosted Runners for org '{}'\n{}",
        owner,
        "-".repeat(50)
    );

    for r in runners.runners {
        let status = if r.busy { "⛔ busy" } else { "✅ idle" };
        println!("{:<25} → {}", r.name, status);
    }

    Ok(())
}
