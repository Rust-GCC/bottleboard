use anyhow::Result;
use structopt::StructOpt;

mod artifact;
mod json;

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short, long, help = "Personal access token if available")]
    token: Option<String>,
    // #[structopt(short, long, help = "Directory in which to cache testsuite results")]
    // cache: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    let archives = artifact::fetch_result_files(args.token).await?;
    for archive in archives {
        let _ = artifact::extract_json(archive).await;
    }

    Ok(())
}
