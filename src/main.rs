use std::path::{Path, PathBuf};
mod tendril;
use tendril::Tendril;

fn main() {
    // const OS: &str = std::env::consts::OS;

    let user_data_folder = std::env::current_dir().unwrap();
    let tendrils_file_path = Path::new(&user_data_folder).join("tendrils.json");
    let tendrils_overrides_file_path = Path::new(&user_data_folder).join("tendrils-overrides.json");

    let global_tendrils = parse_tendrils(&tendrils_file_path).unwrap();
    let local_tendrils = match parse_tendrils(&tendrils_overrides_file_path) {
        Ok(v) => v,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Vec::from([]),
            _ => panic!("{}", e)
        }
    };

    let tendrils = resolve_overrides(&global_tendrils, &local_tendrils);

    print!("{:#?}", tendrils);
}

/// # Arguments
/// - `path` - The path to a file defining
/// Tendril items in JSON format
fn parse_tendrils(path: &PathBuf) -> Result<Vec<Tendril>, std::io::Error> {
    let file_contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => return Err(e),
    };

    Ok(serde_json::from_str(&file_contents).expect("Could not parse JSON."))
}

/// Returns a list of all Tendrils after replacing global ones with any
/// applicable overrides.
/// # Arguments
/// - `global` - The set of Tendril items defined in tendrils.json
/// - `overrides` - The set of Tendril items defined in tendrils-overrides.json
fn resolve_overrides(global: &Vec<Tendril>, overrides: &Vec<Tendril>) -> Vec<Tendril> {
    let mut resolved_tendrils: Vec<Tendril> = Vec::from([]);

    for tendril in global {
        let mut last_index: usize = 0;
        let overrides_iter = overrides.into_iter();

        if overrides_iter.enumerate().any(|(i, x)| { 
            last_index = i;
            x.id() == tendril.id() })
        {
            resolved_tendrils.push(overrides[last_index].clone());
        }
        else {
            resolved_tendrils.push(tendril.clone())
        }
    }

    resolved_tendrils
}
