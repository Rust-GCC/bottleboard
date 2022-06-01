use std::collections::HashMap;

use anyhow::Result;
use json::TestsuiteResult;
use structopt::StructOpt;

mod artifact;
mod json;

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short, long, help = "Personal access token if available")]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    let mut db = HashMap::<String, TestsuiteResult>::new();

    let archives = artifact::fetch_result_files(args.token).await?;
    for archive in archives {
        let bytes = artifact::extract_json(archive).await?;
        let json = TestsuiteResult::from_bytes(bytes.as_slice());

        match json {
            // FIXME: Do we really want to update inconditionally?
            Ok(json) => {
                db.insert(format!("{}-{}", json.name, json.date), json);
            }
            Err(e) => eprintln!("invalid json file... skipping: {}", e),
        }
    }

    Ok(())
}
