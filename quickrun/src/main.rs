use quickrun::cli::cli_run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    cli_run().await?;
    Ok(())
}
