use octocrab::models::RunId;

// FIXME: Add doc
// FIXME: Rename
// FIXME: Probably want that in a method actually
pub async fn fetch_result_files() {
    let instance = octocrab::instance();
    let actions = instance.actions();
    let list = actions.list_workflow_run_artifacts("rust-gcc", "testing", RunId(0));

    if let Some(page) = list.send().await.unwrap().value {
        for artifact in page {
            dbg!(artifact);
        }
    }
}
