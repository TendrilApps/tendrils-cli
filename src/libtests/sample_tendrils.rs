use crate::Tendril;

pub struct SampleTendrils {}

impl SampleTendrils {
    pub fn tendril_1() -> Tendril {
            Tendril {
            group: "MyApp".to_string(),
            name: "settings.json".to_string(),
            parent_dirs_mac: [].to_vec(),
            parent_dirs_windows: ["C:\\Users\\<user>\\AppData\\".to_string()].to_vec(),
            dir_merge: false,
            link: false,
        }
    }

    pub fn tendril_1_json() -> String {
        r#"{
            "group": "MyApp",
            "name": "settings.json",
            "parent-dirs-mac": [],
            "parent-dirs-windows": ["C:\\Users\\<user>\\AppData\\"],
            "dir-merge": false,
            "link": false
        }"#.to_string()
    }

    pub fn tendril_2() -> Tendril {
        Tendril {
            group: "MyApp2".to_string(),
            name: "settings2.json".to_string(),
            parent_dirs_mac: ["some/mac/path".to_string()].to_vec(),
            parent_dirs_windows: ["C:\\Users\\<user>\\Documents\\".to_string()]
                .to_vec(),
            dir_merge: true,
            link: false,
        }
    }

    pub fn tendril_2_json() -> String {
        r#"{
            "group": "MyApp2",
            "name": "settings2.json",
            "parent-dirs-mac": ["some/mac/path"],
            "parent-dirs-windows": ["C:\\Users\\<user>\\Documents\\"],
            "dir-merge": true,
            "link": false
        }"#.to_string()
    }

    pub fn tendril_3() -> Tendril {
        Tendril {
            group: "MyApp".to_string(),
            name: "linkme.txt".to_string(),
            parent_dirs_mac: ["some/mac/path".to_string()].to_vec(),
            parent_dirs_windows: ["C:\\Users\\<user>".to_string()]
                .to_vec(),
            dir_merge: false,
            link: true,
        }
    }

    pub fn tendril_3_json() -> String {
        r#"{
            "group": "MyApp",
            "name": "linkme.txt",
            "parent-dirs-mac": ["some/mac/path"],
            "parent-dirs-windows": ["C:\\Users\\<user>"],
            "dir-merge": false,
            "link": true
        }"#.to_string()
    }

    pub fn build_tendrils_json(json_tendrils: &Vec<String>) -> String {
        let json_chunks:Vec<String> = [
            "[".to_string(),
            json_tendrils.join(","),
            "]".to_string()
        ].to_vec();
        json_chunks.join("")
    }
}
