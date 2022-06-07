mod cache;
mod error;
mod json;

use std::path::PathBuf;

use rocket::{http::Status, State};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Args {
    #[structopt(short, long, help = "Personal access token if available")]
    token: Option<String>,
    #[structopt(short, long, help = "Directory in which to store the results")]
    cache: PathBuf,
}

#[rocket::get("/")]
async fn index(state: &State<Args>) -> Status {
    if let Err(e) = cache::fetch_ci_results(state.inner()).await {
        dbg!(e);
        return Status::NoContent;
    }

    Status::Accepted
}

#[rocket::launch]
async fn rocket() -> _ {
    let args = Args::from_args();

    rocket::build()
        .mount("/", rocket::routes![index])
        .manage(args)
}
