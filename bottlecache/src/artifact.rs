use std::io::{BufReader, Cursor};

// FIXME: Add proper Artifact structure?
pub struct Artifact(Vec<u8>);

// FIXME: Add documentation
pub async fn download_artifact(download_url: &str) -> Result<Artifact, reqwest::Error> {
    let response = reqwest::get(download_url).await?;
    let zip = response.bytes().await?;

    Ok(Artifact(zip.to_vec()))
}

pub async fn extract_json(artifact: Artifact) -> Result<(), zip::result::ZipError> {
    let reader = BufReader::new(Cursor::new(artifact.0));
    let mut zip = zip::ZipArchive::new(reader)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        dbg!(file.name());
        std::io::copy(&mut file, &mut std::io::stdout())?;
    }

    todo!()
}

// FIXME: Add doc
// FIXME: Rename
// FIXME: Probably want that in a method actually
// FIXME: Return the actual files
pub async fn fetch_result_files() -> Result<Vec<String>, octocrab::Error> {
    let instance = octocrab::instance();
    let actions = instance.actions();

    let runs = instance
        .workflows("rust-gcc", "testing")
        .list_runs("nightly_run.yml")
        .send()
        .await?;

    let mut urls = vec![];

    for run in runs {
        let list = actions.list_workflow_run_artifacts("rust-gcc", "testing", run.id);
        if let Some(page) = list.send().await?.value {
            for artifact in page {
                if artifact.name.ends_with(".json") {
                    urls.push(artifact.url.into());
                }
            }
        }
    }

    Ok(urls)
}
