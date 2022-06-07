mod artifact;

use std::path::PathBuf;

use crate::error::Error;
use crate::json::TestsuiteResult;
use crate::Args;

pub async fn fetch_ci_results(args: &Args) -> Result<(), Error> {
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
