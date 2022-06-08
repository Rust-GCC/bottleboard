mod cache;
mod error;
mod json;

use std::path::PathBuf;

use cache::Cache;
use rocket::{http::Status, State};
use structopt::StructOpt;
use tokio::sync::Mutex;

#[derive(StructOpt, Debug)]
pub struct Args {
    #[structopt(short, long, help = "Personal access token if available")]
    token: Option<String>,
    #[structopt(short, long, help = "Directory in which to store the results")]
    cache: PathBuf,
}

#[rocket::get("/")]
async fn index(state: &State<Mutex<Cache>>) -> Status {
    // FIXME: Can we unwrap here?
    // FIXME: Can we improve this error handling?
    let _data = {
        let mut cache = state.inner().lock().await;
        match cache.data().await {
            Err(e) => {
                dbg!(e);
                return Status::NoContent;
            }
            Ok(data) => data,
        }
    };

    Status::Accepted
}

#[rocket::launch]
async fn rocket() -> _ {
    let args = Args::from_args();

    let mut cache = Cache::new(args.token, args.cache);
    cache.data().await.expect("couldn't initialize cache");

    let cache = Mutex::new(cache);

    rocket::build()
        .mount("/", rocket::routes![index])
        .manage(cache)
}
