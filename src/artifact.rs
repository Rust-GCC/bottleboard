// FIXME: Add doc
// FIXME: Rename
// FIXME: Probably want that in a method actually
pub async fn fetch_result_files() {
    let instance = octocrab::instance();
    let actions = instance.actions();

    // FIXME: No unwrap
    // We probably need to limit runs which only happened in the last ~90 days.
    // This should prevent us from running into API rate limits
    let runs = instance
        .workflows("rust-gcc", "testing")
        .list_runs("nightly_run.yml")
        .send()
        .await
        .unwrap();

    for run in runs {
        let list = actions.list_workflow_run_artifacts("rust-gcc", "testing", run.id);
        // FIXME: No unwrap
        if let Some(page) = list.send().await.unwrap().value {
            for artifact in page {
                dbg!(artifact);
            }
        }
    }
}
