use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    binarygrep::run()?;
    Ok(())
}
