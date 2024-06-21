pub type AnyError = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, AnyError>;

pub trait CheckStatus {
    fn check(self) -> std::result::Result<(), String>;
}

impl CheckStatus for &std::process::ExitStatus {
    fn check(self) -> std::result::Result<(), String> {
        if self.success() {
            Ok(())
        } else {
            Err(format!("command did not complete successfully ({self})"))
        }
    }
}
