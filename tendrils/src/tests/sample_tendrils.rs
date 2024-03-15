use crate::TendrilBundle;

pub struct SampleTendrils {}

impl SampleTendrils {
    pub fn tendril_1() -> TendrilBundle {
            TendrilBundle {
            group: "MyApp".to_string(),
            names: vec!["settings.json".to_string()],
            parents: ["C:\\Users\\MyName\\AppData\\".to_string()].to_vec(),
            dir_merge: false,
            link: false,
            profiles: vec![],
        }
    }

    pub fn tendril_1_json() -> String {
        r#"{
            "group": "MyApp",
            "names": "settings.json",
            "parents": ["C:\\Users\\MyName\\AppData\\"],
            "dir-merge": false,
            "link": false,
            "profiles": []
        }"#.to_string()
    }

    pub fn tendril_2() -> TendrilBundle {
        TendrilBundle {
            group: "MyApp2".to_string(),
            names: vec!["settings2.json".to_string()],
            parents: ["some/parent/path".to_string()].to_vec(),
            dir_merge: true,
            link: false,
            profiles: vec!["win".to_string()],
        }
    }

    pub fn tendril_2_json() -> String {
        r#"{
            "group": "MyApp2",
            "names": "settings2.json",
            "parents": ["some/parent/path"],
            "dir-merge": true,
            "link": false,
            "profiles": ["win"]
        }"#.to_string()
    }

    pub fn tendril_3() -> TendrilBundle {
        TendrilBundle {
            group: "MyApp".to_string(),
            names: vec!["linkme.txt".to_string()],
            parents: ["some/parent/path3".to_string()].to_vec(),
            dir_merge: false,
            link: true,
            profiles: vec!["mac".to_string()],
        }
    }

    pub fn tendril_3_json() -> String {
        r#"{
            "group": "MyApp",
            "names": "linkme.txt",
            "parents": ["some/parent/path3"],
            "dir-merge": false,
            "link": true,
            "profiles": ["mac"]
        }"#.to_string()
    }

    pub fn tendril_4() -> TendrilBundle {
        TendrilBundle {
            group: "MyApp".to_string(),
            names: vec!["misc.txt".to_string()],
            parents: ["some/parent/path4".to_string()].to_vec(),
            dir_merge: false,
            link: false,
            profiles: vec!["mac".to_string(), "win".to_string()],
        }
    }

    pub fn tendril_4_json() -> String {
        r#"{
            "group": "MyApp",
            "names": "misc.txt",
            "parents": ["some/parent/path4"],
            "dir-merge": false,
            "link": false,
            "profiles": ["mac", "win"]
        }"#.to_string()
    }

    pub fn tendril_5() -> TendrilBundle {
        TendrilBundle {
            group: "MyApp".to_string(),
            names: vec!["misc.txt".to_string()],
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
            "names": "misc.txt",
            "parents": "some/parent/path5",
            "profiles": "mac"
        }"#.to_string()
    }

    pub fn tendril_6() -> TendrilBundle {
        TendrilBundle {
            group: "MyApp".to_string(),
            names: vec!["misc1.txt".to_string(), "misc2.txt".to_string()],
            parents: ["some/parent/path6a".to_string(), "some/parent/path6b".to_string()].to_vec(),
            dir_merge: false,
            link: false,
            profiles: vec!["mac".to_string(), "win".to_string()],
        }
    }

    pub fn tendril_6_json() -> String {
        r#"{
            "group": "MyApp",
            "names": ["misc1.txt", "misc2.txt"],
            "parents": ["some/parent/path6a", "some/parent/path6b"],
            "profiles": ["mac", "win"]
        }"#.to_string()
    }

    pub fn all_tendrils() -> Vec<TendrilBundle> {
        vec![
            SampleTendrils::tendril_1(),
            SampleTendrils::tendril_2(),
            SampleTendrils::tendril_3(),
            SampleTendrils::tendril_4(),
            SampleTendrils::tendril_5(),
            SampleTendrils::tendril_6(),
        ]
    }

    pub fn all_tendril_jsons() -> Vec<String> {
        vec![
            SampleTendrils::tendril_1_json(),
            SampleTendrils::tendril_2_json(),
            SampleTendrils::tendril_3_json(),
            SampleTendrils::tendril_4_json(),
            SampleTendrils::tendril_5_json(),
            SampleTendrils::tendril_6_json(),
        ]
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
