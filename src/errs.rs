pub type AnyError = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, AnyError>;

pub fn check_status(status: std::process::ExitStatus) -> std::result::Result<(), String> {
    if status.success() {
        Ok(())
    } else {
        Err(format!("command did not complete successfully ({status})"))
    }
}
