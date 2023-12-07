use std::path::Path;
mod tendril;
use tendril::Tendril;

fn main() {
    let user_data_folder = std::env::current_dir().unwrap();
    let tendrils_file_path = Path::new(&user_data_folder).join("tendrils.json");
    let tendril_overrides_file_path = Path::new(&user_data_folder).join("tendrils-overrides.json");

    let tendrils_file_contents = std::fs::read_to_string(&tendrils_file_path)
        .expect(format!("Could not read file at: {:?}", &tendrils_file_path).as_str());

    let tendril_overrides_file_contents = match tendril_overrides_file_path.exists() {
        true => Some(std::fs::read_to_string(&tendril_overrides_file_path)
            .expect(format!("Could not read file at: {:?}", &tendril_overrides_file_path).as_str())),
        false => None
    };

    let global_tendrils = parse_tendrils(&tendrils_file_contents).expect("Could not parse JSON");
    let local_tendrils = match &tendril_overrides_file_contents {
        Some(file_content) => parse_tendrils(file_content).expect("Could not parse JSON"),
        None => [].to_vec()
    };

    if local_tendrils.is_empty() { println!("No local overrides were found.") }

    let tendrils = resolve_overrides(&global_tendrils, &local_tendrils);

    print!("{:#?}", tendrils);
}

/// # Arguments
/// - `json` - Tendril items in JSON format
fn parse_tendrils(json: &str) -> Result<Vec<Tendril>, serde_json::Error> {
    serde_json::from_str::<Vec<Tendril>>(&json)
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

#[cfg(test)]
mod tests {
    use super::parse_tendrils;

    #[test]
    fn parse_tendrils_empty_string_returns_error() {
        let given = "";

        assert!(parse_tendrils(&given).is_err());
    }

    #[test]
    fn parse_tendrils_invalid_json_returns_error() {
        let given = "I'm not JSON";

        assert!(parse_tendrils(&given).is_err());
    }

    #[test]
    fn parse_tendrils_json_missing_field_returns_error() {
        let given = r#"
        [
            {
                "app": "MyApp",
                "parent-dirs-mac": [],
                "parent-dirs-windows": ["C:\\Users\\<user>\\AppData\\"],
                "folder-merge": false
            }
        ]"#;

        assert!(parse_tendrils(&given).is_err());
    }

    #[test]
    fn parse_tendrils_empty_json_array_returns_empty() {
        let given = "[]";
        let expected = [].to_vec();

        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_tendrils_single_tendril_in_json_returns_tendril() {
        let given = r#"
        [
            {
                "app": "MyApp",
                "name": "settings.json",
                "parent-dirs-mac": [],
                "parent-dirs-windows": ["C:\\Users\\<user>\\AppData\\"],
                "folder-merge": false
            }
        ]"#;

        let tendril1 = super::Tendril {
            app: "MyApp".to_string(),
            name: "settings.json".to_string(),
            parent_dirs_mac: [].to_vec(),
            parent_dirs_windows: ["C:\\Users\\<user>\\AppData\\".to_string()].to_vec(),
            folder_merge: false
        };

        let expected = [tendril1].to_vec();

        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_tendrils_multiple_tendrils_in_json_returns_tendrils() {
        let given = r#"
        [
            {
                "app": "MyApp",
                "name": "settings.json",
                "parent-dirs-mac": [],
                "parent-dirs-windows": ["C:\\Users\\<user>\\AppData\\"],
                "folder-merge": false
            },
            {
                "app": "MyApp2",
                "name": "settings2.json",
                "parent-dirs-mac": ["some/mac/path"],
                "parent-dirs-windows": ["C:\\Users\\<user>\\Documents\\"],
                "folder-merge": true
            }
        ]"#;

        let tendril1 = super::Tendril {
            app: "MyApp".to_string(),
            name: "settings.json".to_string(),
            parent_dirs_mac: [].to_vec(),
            parent_dirs_windows: ["C:\\Users\\<user>\\AppData\\".to_string()].to_vec(),
            folder_merge: false
        };

        let tendril2 = super::Tendril {
            app: "MyApp2".to_string(),
            name: "settings2.json".to_string(),
            parent_dirs_mac: ["some/mac/path".to_string()].to_vec(),
            parent_dirs_windows: ["C:\\Users\\<user>\\Documents\\".to_string()].to_vec(),
            folder_merge: true
        };

        let expected = [tendril1, tendril2].to_vec();

        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_tendrils_ignores_extra_json_field_returns_tendril() {
        let given = r#"
        [
            {
                "some-extra-field": "ABCD",
                "app": "MyApp",
                "name": "settings.json",
                "parent-dirs-mac": [],
                "parent-dirs-windows": ["C:\\Users\\<user>\\AppData\\"],
                "folder-merge": false
            }
        ]"#;

        let tendril1 = super::Tendril {
            app: "MyApp".to_string(),
            name: "settings.json".to_string(),
            parent_dirs_mac: [].to_vec(),
            parent_dirs_windows: ["C:\\Users\\<user>\\AppData\\".to_string()].to_vec(),
            folder_merge: false
        };

        let expected = [tendril1].to_vec();
        let actual = parse_tendrils(&given).unwrap();

        assert_eq!(actual, expected);
    }
}
