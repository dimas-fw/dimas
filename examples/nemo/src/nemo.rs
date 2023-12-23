//! The desktop UI for nemo, a network monitoring toolset based on DiMAS
//! Copyright © 2023 Stephan Kunz

use nemo::app::app_main;

// this stub is necessary because some platforms require building
// as dll (mobile / wasm) and some require to be built as executable
// unfortunately cargo doesn't facilitate this without a main.rs stub
#[tokio::main]
async fn main() {
    app_main();
}
