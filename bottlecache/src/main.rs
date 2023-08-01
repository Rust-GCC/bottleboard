mod cache;
mod error;

use std::collections::HashSet;
use std::path::PathBuf;

use cache::Cache;
use chrono::NaiveDate;
use itertools::Itertools;
use rocket::{request::FromParam, serde::json::Json, State};
use structopt::StructOpt;
use tokio::sync::Mutex;

use common::TestsuiteResult;

#[derive(StructOpt, Debug)]
pub struct Args {
    #[structopt(short, long, help = "Personal access token")]
    token: String,
    #[structopt(short, long, help = "Location in which to store the cached JSON files")]
    cache: Option<PathBuf>,
    #[structopt(
        long,
        help = "Create a mocked instance of bottlecache, which only serves runs from disk"
    )]
    mock: bool,
}

struct NaiveDateRequest(NaiveDate);

#[derive(Debug)]
struct NaiveDateError;

impl<'r> FromParam<'r> for NaiveDateRequest {
    type Error = chrono::ParseError;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        // FIXME: What format to use here? How to enforce it?
        let date = NaiveDate::parse_from_str(param, "%Y-%m-%d")?;

        Ok(NaiveDateRequest(date))
    }
}

#[rocket::get("/api/testsuites/<key>")]
async fn testsuite_by_key(state: &State<Mutex<Cache>>, key: &str) -> Json<Vec<TestsuiteResult>> {
    // FIXME: Can we unwrap here?
    // FIXME: Can we improve this error handling?
    let mut cache = state.inner().lock().await;
    let data = cache.data().await.expect("could not fetch data");

    Json(data.into_iter().filter(|json| json.name == key).collect())
}

#[rocket::get("/api/runs/<date>")]
async fn runs_by_date(
    state: &State<Mutex<Cache>>,
    date: NaiveDateRequest,
) -> Json<Vec<TestsuiteResult>> {
    // FIXME: Can we unwrap here?
    // FIXME: Can we improve this error handling?
    let mut cache = state.inner().lock().await;
    let runs = cache.data().await.expect("could not fetch data");

    // There is only one run per testsuite per day
    Json(
        runs.into_iter()
            .filter(|json| json.date == date.0)
            .collect(),
    )
}

#[rocket::get("/api/dates")]
async fn all_run_dates(state: &State<Mutex<Cache>>) -> Json<HashSet<NaiveDate>> {
    let mut cache = state.inner().lock().await;
    let runs = cache.data().await.expect("could not fetch data");

    Json(runs.into_iter().fold(HashSet::new(), |mut set, run| {
        set.insert(run.date);
        set
    }))
}

#[rocket::get("/api/testsuites/<key>/<date>")]
async fn testsuite_by_key_date(
    state: &State<Mutex<Cache>>,
    key: &str,
    date: NaiveDateRequest,
) -> Json<Option<TestsuiteResult>> {
    // FIXME: Can we unwrap here?
    // FIXME: Can we improve this error handling?
    let mut cache = state.inner().lock().await;
    let runs = cache.data().await.expect("could not fetch data");

    Json(
        runs.into_iter()
            .find(|json| json.name == key && json.date == date.0),
    )
}

#[rocket::get("/api/testsuites")]
async fn testsuites(state: &State<Mutex<Cache>>) -> Json<Vec<String>> {
    // FIXME: Can we unwrap here?
    // FIXME: Can we improve this error handling?
    let mut cache = state.inner().lock().await;
    let data = cache.data().await.expect("could not fetch data");

    Json(data.into_iter().map(|run| run.name).unique().collect())
}

#[rocket::launch]
async fn rocket() -> _ {
    let args = Args::from_args();
    env_logger::init();

    let mut cache =
        Cache::try_new(args.token, args.cache, args.mock).expect("couldn't create cache");
    cache.data().await.expect("couldn't fetch initial cache");

    let cache = Mutex::new(cache);

    // FIXME: Should we unwrap here?
    let cors = rocket_cors::CorsOptions::default().to_cors().unwrap();

    rocket::build()
        .attach(cors)
        .mount(
            "/",
            rocket::routes![
                testsuites,
                testsuite_by_key,
                runs_by_date,
                all_run_dates,
                testsuite_by_key_date
            ],
        )
        .manage(cache)
}
