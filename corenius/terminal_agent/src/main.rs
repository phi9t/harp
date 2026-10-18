#[tokio::main]
async fn main() -> anyhow::Result<()> {
    corenius::cli::run().await
}
