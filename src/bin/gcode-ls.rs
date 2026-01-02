use anyhow::Result;
use gcode_language_server::lsp::server::serve;

#[tokio::main]
async fn main() -> Result<()> {
    serve().await
}
