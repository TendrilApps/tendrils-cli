use crate::TendrilBundle;

pub struct SampleTendrils {}

impl SampleTendrils {
    pub fn tendril_1() -> TendrilBundle {
        TendrilBundle {
            local: "settings.json".to_string(),
            remotes: ["C:\\Users\\MyName\\AppData\\settings.json".to_string()].to_vec(),
            dir_merge: false,
            link: false,
            profiles: vec![],
        }
    }

    pub fn tendril_1_json() -> String {
        r#"{
            "local": "settings.json",
            "remotes": ["C:\\Users\\MyName\\AppData\\settings.json"],
            "dir-merge": false,
            "link": false,
            "profiles": []
        }"#
        .to_string()
    }

    pub fn tendril_2() -> TendrilBundle {
        TendrilBundle {
            local: "SomeApp2/settings2.json".to_string(),
            remotes: ["some/remote/path/settings2.json".to_string()].to_vec(),
            dir_merge: true,
            link: false,
            profiles: vec!["win".to_string()],
        }
    }

    pub fn tendril_2_json() -> String {
        r#"{
            "local": "SomeApp2/settings2.json",
            "remotes": ["some/remote/path/settings2.json"],
            "dir-merge": true,
            "link": false,
            "profiles": ["win"]
        }"#
        .to_string()
    }

    pub fn tendril_3() -> TendrilBundle {
        TendrilBundle {
            local: "SomeApp/linkme.txt".to_string(),
            remotes: ["some/remote/path3/linkme.txt".to_string()].to_vec(),
            dir_merge: false,
            link: true,
            profiles: vec!["mac".to_string()],
        }
    }

    pub fn tendril_3_json() -> String {
        r#"{
            "local": "SomeApp/linkme.txt",
            "remotes": ["some/remote/path3/linkme.txt"],
            "dir-merge": false,
            "link": true,
            "profiles": ["mac"]
        }"#
        .to_string()
    }

    pub fn tendril_4() -> TendrilBundle {
        TendrilBundle {
            local: "SomeApp/localName.txt".to_string(),
            remotes: ["some/remote/path4/remoteName.txt".to_string()].to_vec(),
            dir_merge: false,
            link: false,
            profiles: vec!["mac".to_string(), "win".to_string()],
        }
    }

    pub fn tendril_4_json() -> String {
        r#"{
            "local": "SomeApp/localName.txt",
            "remotes": ["some/remote/path4/remoteName.txt"],
            "dir-merge": false,
            "link": false,
            "profiles": ["mac", "win"]
        }"#
        .to_string()
    }

    pub fn tendril_5() -> TendrilBundle {
        TendrilBundle {
            local: "misc.txt".to_string(),
            remotes: ["some/remote/path5/misc.txt".to_string()].to_vec(),
            dir_merge: false,
            link: false,
            profiles: vec!["mac".to_string()],
        }
    }

    /// Minimal schema with single values passed as strings
    /// instead of arrays
    pub fn tendril_5_json() -> String {
        r#"{
            "local": "misc.txt",
            "remotes": "some/remote/path5/misc.txt",
            "profiles": "mac"
        }"#
        .to_string()
    }

    pub fn tendril_6() -> TendrilBundle {
        TendrilBundle {
            local: "SomeApp/misc.txt".to_string(),
            remotes: [
                "some/remote/path6a/misc1.txt".to_string(),
                "some/remote/path6b/misc2.txt".to_string(),
            ]
            .to_vec(),
            dir_merge: false,
            link: false,
            profiles: vec!["mac".to_string(), "win".to_string()],
        }
    }

    pub fn tendril_6_json() -> String {
        r#"{
            "local": "SomeApp/misc.txt",
            "remotes": [
                "some/remote/path6a/misc1.txt",
                "some/remote/path6b/misc2.txt"
            ],
            "profiles": ["mac", "win"]
        }"#
        .to_string()
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

    pub fn build_tendrils_json(json_tendrils: &[String]) -> String {
        let json_chunks: Vec<String> =
            ["{\"tendrils\": [".to_string(), json_tendrils.join(","), "]}".to_string()]
                .to_vec();
        json_chunks.join("")
    }
}
