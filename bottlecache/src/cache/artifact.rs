use std::io::{BufReader, Cursor};

use octocrab::{
    actions::ActionsHandler, models::ArtifactId, params::actions::ArchiveFormat, OctocrabBuilder,
};

// FIXME: Add proper Artifact structure?
// FIXME: Is that type even needed?
#[derive(Debug)]
pub struct Archive(Vec<u8>);

// FIXME: Add documentation
pub async fn download_artifact(
    instance: &ActionsHandler<'_>,
    artifact: ArtifactId,
) -> Result<Archive, octocrab::Error> {
    let archive = instance
        .download_artifact("rust-gcc", "testing", artifact, ArchiveFormat::Zip)
        .await?;

    Ok(Archive(archive.to_vec()))
}

pub async fn extract_json(artifact: Archive) -> Result<Vec<u8>, zip::result::ZipError> {
    let reader = BufReader::new(Cursor::new(artifact.0));
    let mut zip = zip::ZipArchive::new(reader)?;

    // We're always looking for the first file
    let mut file = zip.by_index(0)?;
    let mut bytes = vec![];
    std::io::copy(&mut file, &mut bytes)?;

    Ok(bytes)
}

// FIXME: Add doc
// FIXME: Rename
// FIXME: Probably want that in a method actually
// FIXME: Return the actual files
pub async fn fetch_result_files(
    access_token: Option<String>,
) -> Result<Vec<Archive>, octocrab::Error> {
    let builder = OctocrabBuilder::new();
    let builder = match access_token {
        None => builder,
        Some(tok) => builder.personal_token(tok),
    };

    let instance = builder.build()?;
    let actions = instance.actions();

    let runs = instance
        .workflows("rust-gcc", "testing")
        .list_runs("nightly_run.yml")
        .send()
        .await?;

    let mut archives = vec![];

    for run in runs {
        let list = actions.list_workflow_run_artifacts("rust-gcc", "testing", run.id);
        if let Some(page) = list.send().await?.value {
            for artifact in page {
                if artifact.name.ends_with(".json") {
                    archives.push(download_artifact(&actions, artifact.id).await?);
                }
            }
        }
    }

    Ok(archives)
}
