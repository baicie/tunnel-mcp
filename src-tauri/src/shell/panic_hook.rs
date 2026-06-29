use std::sync::Once;

static INSTALL: Once = Once::new();

pub fn install_panic_hook() {
    INSTALL.call_once(|| {
        std::panic::set_hook(Box::new(|panic_info| {
            eprintln!("desktop shell panic: {panic_info}");
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_panic_hook_should_be_idempotent() {
        install_panic_hook();
        install_panic_hook();
    }
}
