use std::path::PathBuf;

use json::TestsuiteResult;
use rocket::{http::Status, State};
use structopt::StructOpt;

mod artifact;
mod json;

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short, long, help = "Personal access token if available")]
    token: Option<String>,
    #[structopt(short, long, help = "Directory in which to store the results")]
    cache: PathBuf,
}

// FIXME: Move in its own module
#[derive(Debug)]
enum Error {
    GitHub,
    Unzipping,
    Disk,
}

impl From<octocrab::Error> for Error {
    fn from(_: octocrab::Error) -> Self {
        Error::GitHub
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(_: zip::result::ZipError) -> Self {
        Error::Unzipping
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Error::Disk
    }
}

async fn fetch_ci_results(args: &Args) -> Result<(), Error> {
    let token = args.token.clone();
    let archives = artifact::fetch_result_files(token).await?;
    for archive in archives {
        let bytes = artifact::extract_json(archive).await?;
        let json = TestsuiteResult::from_bytes(bytes.as_slice());

        match json {
            Ok(json) => {
                let path = args
                    .cache
                    .join(PathBuf::from(format!("{}-{}", json.name, json.date)));
                dbg!(&path);
                eprintln!("valid json! Writing to `{}`", path.display());
                json.write_to(&path)?;
            }
            Err(e) => eprintln!("invalid json file... skipping it. Reason: `{}`", e),
        }
    }

    Ok(())
}

#[rocket::get("/")]
async fn index(state: &State<Args>) -> Status {
    if let Err(e) = fetch_ci_results(state.inner()).await {
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
