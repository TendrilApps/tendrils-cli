use crate::RawTendril;
use crate::enums::TendrilMode;

pub struct SampleTendrils {}

impl SampleTendrils {
    pub fn raw_tendrils_1() -> Vec<RawTendril> {
        vec![
            RawTendril {
                local: "settings.json".to_string(),
                remote: "C:\\Users\\MyName\\AppData\\settings.json".to_string(),
                mode: TendrilMode::Overwrite,
                profiles: vec![],
            }
        ]
    }

    pub fn tendril_1_json() -> String {
        r#""settings.json": [
            {
                "remotes": ["C:\\Users\\MyName\\AppData\\settings.json"],
                "dir-merge": false,
                "link": false,
                "profiles": []
            }
        ]"#
        .to_string()
    }

    pub fn raw_tendrils_2() -> Vec<RawTendril> {
        vec![
            RawTendril {
                local: "SomeApp2/settings2.json".to_string(),
                remote: "some/remote/path/settings2.json".to_string(),
                mode: TendrilMode::Merge,
                profiles: vec!["win".to_string()],
            }
        ]
    }

    pub fn tendril_2_json() -> String {
        r#""SomeApp2/settings2.json": {
            "remotes": ["some/remote/path/settings2.json"],
            "dir-merge": true,
            "link": false,
            "profiles": ["win"]
        }"#
        .to_string()
    }

    pub fn raw_tendrils_3() -> Vec<RawTendril> {
        vec![
            RawTendril {
                local: "SomeApp/linkme.txt".to_string(),
                remote: "some/remote/path3/linkme.txt".to_string(),
                mode: TendrilMode::Link,
                profiles: vec!["mac".to_string()],
            }
        ]
    }

    pub fn tendril_3_json() -> String {
        r#""SomeApp/linkme.txt": {
            "remotes": ["some/remote/path3/linkme.txt"],
            "dir-merge": false,
            "link": true,
            "profiles": ["mac"]
        }"#
        .to_string()
    }

    pub fn raw_tendrils_4() -> Vec<RawTendril> {
        vec![
            RawTendril {
                local: "SomeApp/localName.txt".to_string(),
                remote: "some/remote/path4/remoteName.txt".to_string(),
                mode: TendrilMode::Overwrite,
                profiles: vec!["mac".to_string(), "win".to_string()],
            }
        ]
    }

    pub fn tendril_4_json() -> String {
        r#""SomeApp/localName.txt": {
            "remotes": ["some/remote/path4/remoteName.txt"],
            "dir-merge": false,
            "link": false,
            "profiles": ["mac", "win"]
        }"#
        .to_string()
    }

    pub fn raw_tendrils_5() -> Vec<RawTendril> {
        vec![
            RawTendril {
                local: "misc.txt".to_string(),
                remote: "some/remote/path5/misc.txt".to_string(),
                mode: TendrilMode::Overwrite,
                profiles: vec!["mac".to_string()],
            }
        ]
    }

    /// Minimal schema with single values passed as strings
    /// instead of arrays
    pub fn tendril_5_json() -> String {
        r#""misc.txt": {
            "remotes": "some/remote/path5/misc.txt",
            "profiles": "mac"
        }"#
        .to_string()
    }

    pub fn raw_tendrils_6() -> Vec<RawTendril> {
        vec![
            RawTendril {
                local: "SomeApp/misc.txt".to_string(),
                remote: "some/remote/path6a/misc1.txt".to_string(),
                mode: TendrilMode::Overwrite,
                profiles: vec!["mac".to_string(), "win".to_string()],
            },
            RawTendril {
                local: "SomeApp/misc.txt".to_string(),
                remote: "some/remote/path6b/misc2.txt".to_string(),
                mode: TendrilMode::Overwrite,
                profiles: vec!["mac".to_string(), "win".to_string()],
            }            
        ]
    }

    pub fn tendril_6_json() -> String {
        r#""SomeApp/misc.txt": {
            "remotes": [
                "some/remote/path6a/misc1.txt",
                "some/remote/path6b/misc2.txt"
            ],
            "profiles": ["mac", "win"]
        }"#
        .to_string()
    }

    pub fn raw_tendrils_7() -> Vec<RawTendril> {
        vec![
            RawTendril {
                local: "host-specific.txt".to_string(),
                remote: "~/host1/specific/path/host1.txt".to_string(),
                mode: TendrilMode::Overwrite,
                profiles: vec!["host1".to_string()],
            },
            RawTendril {
                local: "host-specific.txt".to_string(),
                remote: "~/host2/specific/path/host2.txt".to_string(),
                mode: TendrilMode::Link,
                profiles: vec!["host2".to_string()],
            }
        ]
    }

    pub fn tendril_7_json() -> String {
        r#""host-specific.txt": [
            {
                "remotes": ["~/host1/specific/path/host1.txt"],
                "dir-merge": false,
                "link": false,
                "profiles": ["host1"]
            },
            {
                "remotes": "~/host2/specific/path/host2.txt",
                "dir-merge": false,
                "link": true,
                "profiles": "host2"
            }
        ]"#
        .to_string()
    }

    pub fn all_tendrils() -> Vec<RawTendril> {
        let mut vec = SampleTendrils::raw_tendrils_1();
        vec.append(&mut SampleTendrils::raw_tendrils_2());
        vec.append(&mut SampleTendrils::raw_tendrils_3());
        vec.append(&mut SampleTendrils::raw_tendrils_4());
        vec.append(&mut SampleTendrils::raw_tendrils_5());
        vec.append(&mut SampleTendrils::raw_tendrils_6());
        vec.append(&mut SampleTendrils::raw_tendrils_7());
        vec
    }

    pub fn all_tendril_jsons() -> Vec<String> {
        vec![
            SampleTendrils::tendril_1_json(),
            SampleTendrils::tendril_2_json(),
            SampleTendrils::tendril_3_json(),
            SampleTendrils::tendril_4_json(),
            SampleTendrils::tendril_5_json(),
            SampleTendrils::tendril_6_json(),
            SampleTendrils::tendril_7_json(),
        ]
    }

    pub fn build_tendrils_json(json_tendrils: &[String]) -> String {
        let json_chunks: Vec<String> =
            ["{\"tendrils\": {".to_string(), json_tendrils.join(","), "}}".to_string()]
                .to_vec();
        json_chunks.join("")
    }
}
