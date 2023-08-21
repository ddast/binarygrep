use binarygrep::BgrepError;

fn main() -> Result<(), BgrepError> {
    binarygrep::run()?;
    Ok(())
}
