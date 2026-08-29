use crate::cli::ListFormat;
use anstream::println;
use anyhow::Result;

pub fn run(format: ListFormat) -> Result<()> {
    let root = crate::config::find_root(&std::env::current_dir()?)?;
    let config = crate::config::load(&root)?;
    match format {
        ListFormat::Plain => {
            for name in config.modules.keys() {
                println!("{name}");
            }
        }
        ListFormat::Json => {
            let names: Vec<&String> = config.modules.keys().collect();
            println!("{}", serde_json::to_string(&names)?);
        }
    }
    Ok(())
}
