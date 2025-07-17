// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::gh_api::GitHubClient;
use crate::models::{NewRunnersSaturation, RunnersSaturationScheme};
use crate::schema::runners_saturation::dsl::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use diesel::dsl::count_star;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;

#[async_trait]
pub trait Fetcher: Send {
    async fn fetch(&mut self) -> Result<i64>;
}

pub trait Storage {
    fn put(&mut self, key: &str, data: Box<[u8]>) -> std::io::Result<()>;
    fn get(&self, key: &str) -> Arc<[u8]>;
    fn delete(&self, key: &str) -> std::io::Result<()>;
}

pub struct RunnersSaturation {
    ghc: GitHubClient,
    client: PgConnection,
    state: HashMap<i64, bool>,
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

impl RunnersSaturation {
    pub fn new(ghc: GitHubClient, mut client: PgConnection) -> Self {
        use crate::schema::runners_saturation::dsl::*;
        use diesel::prelude::*;

        let results = runners_saturation
            .distinct_on(rid)
            .order_by((rid, created_at.desc()))
            .select((rid, busy))
            .load::<(i64, bool)>(&mut client)
            .expect("Error loading data");

        return Self {
            ghc,
            client,
            state: results.into_iter().collect(),
        };
    }
}

#[async_trait]
impl Fetcher for RunnersSaturation {
    async fn fetch(&mut self) -> Result<i64> {
        use crate::schema::runners_saturation;

        let runners = self.ghc.runners().await?;
        let now = Utc::now().naive_utc();
        let mut count: i64 = runners_saturation
            .select(count_star())
            .first(&mut self.client)?;
        for r in runners.runners {
            let changed = match self.state.get(&r.id) {
                Some(&old_value) => {
                    if old_value != r.busy {
                        self.state.insert(r.id, r.busy);
                        true
                    } else {
                        false
                    }
                }
                None => {
                    self.state.insert(r.id, r.busy);
                    true
                }
            };

            if changed {
                diesel::insert_into(runners_saturation::table)
                    .values(&NewRunnersSaturation {
                        rid: r.id,
                        name: r.name,
                        busy: r.busy,
                        created_at: now,
                    })
                    .get_result::<RunnersSaturationScheme>(&mut self.client)
                    .expect("Error saving new post");
                count = runners_saturation
                    .select(count_star())
                    .first(&mut self.client)?;
            }
        }
        Ok(count)
    }
}
