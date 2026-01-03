use anyhow::Result;
use std::fs;
use std::path::Path;
use tokio::io::{stdin, stdout};
use tower_lsp::{LspService, Server};

use crate::flavor::registry::FlavorRegistry;
use crate::lsp::backend::Backend;
use crate::Config;

/// Start the LSP server
pub async fn serve() -> Result<()> {
    let config = Config::from_args_and_env()?;

    // Initialize flavor registry with embedded Prusa flavor
    let mut flavor_registry = FlavorRegistry::new();
    flavor_registry.add_embedded_prusa_flavor();

    // Set active flavor from config or default to "prusa"
    let active_flavor = config
        .get_effective_flavor()
        .unwrap_or_else(|| "prusa".to_string());
    flavor_registry.set_active_flavor(&active_flavor);

    // Write embedded flavor to user's config directory for easy access
    if let Err(e) = write_embedded_flavor_to_disk() {
        log::warn!("Failed to write embedded flavor to disk: {}", e);
    }

    let (service, socket) =
        LspService::build(move |client| Backend::new(client, config.clone(), flavor_registry))
            .finish();

    Server::new(stdin(), stdout(), socket).serve(service).await;

    Ok(())
}

/// Write the embedded Prusa flavor to ~/.gcode-ls/flavors/ for user access
fn write_embedded_flavor_to_disk() -> Result<()> {
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;

    let flavor_dir = Path::new(&home_dir).join(".gcode-ls").join("flavors");
    fs::create_dir_all(&flavor_dir)?;

    let flavor_path = flavor_dir.join("prusa.gcode-flavor.toml");

    // Only write if file doesn't exist (don't overwrite user modifications)
    if !flavor_path.exists() {
        let embedded_content = include_str!("../../resources/flavors/prusa.gcode-flavor.toml");
        fs::write(&flavor_path, embedded_content)?;
        log::info!("Created Prusa flavor file: {:?}", flavor_path);
    }

    Ok(())
}
