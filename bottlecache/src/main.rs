use std::path::PathBuf;

use anyhow::Result;
use json::TestsuiteResult;
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

async fn fetch_ci_results(args: Args) -> Result<()> {
    let archives = artifact::fetch_result_files(args.token).await?;
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    fetch_ci_results(args).await
}
