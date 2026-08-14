#[path = "../js_repl.rs"]
mod js_repl;

/// Run the dedicated stdio transport without initializing the PengPilot GUI.
fn main() {
    if let Err(error) = js_repl::serve_stdio() {
        eprintln!("PengPilot JavaScript REPL: {error:#}");
        std::process::exit(1);
    }
}
