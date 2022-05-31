use anyhow::Result;

mod artifact;

#[tokio::main]
async fn main() -> Result<()> {
    let urls = artifact::fetch_result_files().await?;
    for url in urls {
        dbg!(&url);
        let artifact = artifact::download_artifact(&url).await?;
        artifact::extract_json(artifact).await?;
    }

    Ok(())
}
