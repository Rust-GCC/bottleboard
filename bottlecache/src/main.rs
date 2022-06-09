mod cache;
mod error;

use cache::Cache;
use chrono::NaiveDate;
use rocket::{request::FromParam, serde::json::Json, State};
use structopt::StructOpt;
use tokio::sync::Mutex;

use common::TestsuiteResult;

#[derive(StructOpt, Debug)]
pub struct Args {
    #[structopt(short, long, help = "Personal access token if available")]
    token: Option<String>,
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

#[rocket::get("/api/testsuites/<key>/<date>")]
async fn testsuite_by_key_date(
    state: &State<Mutex<Cache>>,
    key: &str,
    date: NaiveDateRequest,
) -> Json<Option<TestsuiteResult>> {
    // FIXME: Can we unwrap here?
    // FIXME: Can we improve this error handling?
    let mut cache = state.inner().lock().await;
    let data = cache.data().await.expect("could not fetch data");

    Json(
        data.into_iter()
            .find(|json| json.name == key && json.date == date.0),
    )
}

#[rocket::get("/api/testsuites")]
async fn testsuites(state: &State<Mutex<Cache>>) -> Json<Vec<TestsuiteResult>> {
    // FIXME: Can we unwrap here?
    // FIXME: Can we improve this error handling?
    let mut cache = state.inner().lock().await;
    let data = cache.data().await.expect("could not fetch data");

    Json(data.into_iter().collect())
}

#[rocket::launch]
async fn rocket() -> _ {
    let args = Args::from_args();

    let mut cache = Cache::try_new(args.token).expect("couldn't create cache");
    cache.data().await.expect("couldn't fetch initial cache");

    let cache = Mutex::new(cache);

    rocket::build()
        .mount(
            "/",
            rocket::routes![testsuites, testsuite_by_key, testsuite_by_key_date],
        )
        .manage(cache)
}
