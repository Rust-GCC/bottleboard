use anyhow::Result;

mod artifact;

#[tokio::main]
async fn main() -> Result<()> {
    artifact::fetch_result_files().await?;

    Ok(())
}
