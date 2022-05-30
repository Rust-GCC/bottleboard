mod artifact;

#[tokio::main]
async fn main() {
    artifact::fetch_result_files().await;
}
