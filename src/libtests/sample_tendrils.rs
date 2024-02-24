use crate::Tendril;

pub struct SampleTendrils {}

impl SampleTendrils {
    pub fn tendril_1() -> Tendril {
            Tendril {
            group: "MyApp".to_string(),
            name: "settings.json".to_string(),
            parents: ["C:\\Users\\MyName\\AppData\\".to_string()].to_vec(),
            dir_merge: false,
            link: false,
            profiles: vec![],
        }
    }

    pub fn tendril_1_json() -> String {
        r#"{
            "group": "MyApp",
            "name": "settings.json",
            "parents": ["C:\\Users\\MyName\\AppData\\"],
            "dir-merge": false,
            "link": false,
            "profiles": []
        }"#.to_string()
    }

    pub fn tendril_2() -> Tendril {
        Tendril {
            group: "MyApp2".to_string(),
            name: "settings2.json".to_string(),
            parents: ["some/parent/path".to_string()].to_vec(),
            dir_merge: true,
            link: false,
            profiles: vec!["win".to_string()],
        }
    }

    pub fn tendril_2_json() -> String {
        r#"{
            "group": "MyApp2",
            "name": "settings2.json",
            "parents": ["some/parent/path"],
            "dir-merge": true,
            "link": false,
            "profiles": ["win"]
        }"#.to_string()
    }

    pub fn tendril_3() -> Tendril {
        Tendril {
            group: "MyApp".to_string(),
            name: "linkme.txt".to_string(),
            parents: ["some/parent/path3".to_string()].to_vec(),
            dir_merge: false,
            link: true,
            profiles: vec!["mac".to_string()],
        }
    }

    pub fn tendril_3_json() -> String {
        r#"{
            "group": "MyApp",
            "name": "linkme.txt",
            "parents": ["some/parent/path3"],
            "dir-merge": false,
            "link": true,
            "profiles": ["mac"]
        }"#.to_string()
    }

    pub fn tendril_4() -> Tendril {
        Tendril {
            group: "MyApp".to_string(),
            name: "misc.txt".to_string(),
            parents: ["some/parent/path4".to_string()].to_vec(),
            dir_merge: false,
            link: false,
            profiles: vec!["mac".to_string(), "win".to_string()],
        }
    }
    pub fn tendril_4_json() -> String {
        r#"{
            "group": "MyApp",
            "name": "misc.txt",
            "parents": ["some/parent/path4"],
            "dir-merge": false,
            "link": false,
            "profiles": ["mac", "win"]
        }"#.to_string()
    }

    pub fn tendril_5() -> Tendril {
        Tendril {
            group: "MyApp".to_string(),
            name: "misc.txt".to_string(),
            parents: ["some/parent/path5".to_string()].to_vec(),
            dir_merge: false,
            link: false,
            profiles: vec!["mac".to_string()],
        }
    }

    /// Minimal schema with single values passed as strings
    /// instead of arrays
    pub fn tendril_5_json() -> String {
        r#"{
            "group": "MyApp",
            "name": "misc.txt",
            "parents": "some/parent/path5",
            "profiles": "mac"
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
