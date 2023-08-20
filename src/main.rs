use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let result = binarygrep::run();
    if let Err(e) = result {
        println!("{}", e);
    }
    Ok(())
}
