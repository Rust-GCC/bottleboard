use std::io::{BufReader, Cursor};

use octocrab::{
    actions::ActionsHandler,
    models::{ArtifactId, RunId},
    params::actions::ArchiveFormat,
    Octocrab, OctocrabBuilder,
};

// FIXME: Add proper Artifact structure?
// FIXME: Is that type even needed?
#[derive(Debug)]
pub struct Archive(Vec<u8>);

pub struct Fetcher {
    instance: Octocrab,
}

// FIXME: Add documentation
async fn download_artifact(
    instance: &ActionsHandler<'_>,
    artifact: ArtifactId,
) -> Result<Archive, octocrab::Error> {
    let archive = instance
        .download_artifact("rust-gcc", "testing", artifact, ArchiveFormat::Zip)
        .await?;

    Ok(Archive(archive.to_vec()))
}

impl Fetcher {
    pub fn try_new(access_token: Option<String>) -> Result<Fetcher, octocrab::Error> {
        let builder = OctocrabBuilder::new();
        let builder = match access_token {
            None => builder,
            Some(tok) => builder.personal_token(tok),
        };

        let instance = builder.build()?;

        Ok(Fetcher { instance })
    }

    // FIXME: Add doc
    pub async fn runs(&self) -> Result<Vec<RunId>, octocrab::Error> {
        let page = self
            .instance
            .workflows("rust-gcc", "testing")
            .list_runs("nightly_run.yml")
            .send()
            .await?;

        Ok(page.into_iter().map(|run| run.id).collect())
    }

    // FIXME: Add doc
    // FIXME: Return the actual files
    pub async fn result_files(
        &self,
        runs: &[RunId],
    ) -> Result<Vec<(RunId, Archive)>, octocrab::Error> {
        let actions = self.instance.actions();
        let mut archives = vec![];

        for run in runs {
            let list = actions.list_workflow_run_artifacts("rust-gcc", "testing", *run);
            if let Some(page) = list.send().await?.value {
                for artifact in page {
                    if artifact.name.ends_with(".json") {
                        archives.push((*run, download_artifact(&actions, artifact.id).await?));
                    }
                }
            }
        }

        Ok(archives)
    }
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
