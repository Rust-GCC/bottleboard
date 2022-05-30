// FIXME: Add doc
// FIXME: Rename
// FIXME: Probably want that in a method actually
// FIXME: Return the actual files
pub async fn fetch_result_files() -> Result<(), octocrab::Error> {
    let instance = octocrab::instance();
    let actions = instance.actions();

    let runs = instance
        .workflows("rust-gcc", "testing")
        .list_runs("nightly_run.yml")
        .send()
        .await?;

    for run in runs {
        let list = actions.list_workflow_run_artifacts("rust-gcc", "testing", run.id);
        if let Some(page) = list.send().await?.value {
            for artifact in page {
                dbg!(artifact);
            }
        }
    }

    Ok(())
}
