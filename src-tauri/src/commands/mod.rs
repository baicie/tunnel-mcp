pub mod app;
pub mod settings;
pub mod shell;

// Product modules live below the shell surface. They follow the same
// command-adapter convention but are wired through `lib.rs` into the
// same `invoke_handler` so that the shell template stays free of
// tunnel / mcp business logic.
pub mod tunnel;
