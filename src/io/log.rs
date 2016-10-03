use nes::nes::NESRuntimeOptions;

/// Logs a message to stdout with a given prefix if the emulator was started
/// with the verbose flag set.
pub fn log<P, T>(prefix: P, text: T, runtime_options: &NESRuntimeOptions) where P: Into<String>, T: Into<String> {
    if runtime_options.verbose {
        println!("[{}] {}", prefix.into(), text.into());
    }
}
