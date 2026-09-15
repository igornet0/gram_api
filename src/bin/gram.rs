//! gram — control-plane CLI for gram_api.

#[tokio::main]
async fn main() -> std::process::ExitCode {
    gram_api::cli::run().await
}
