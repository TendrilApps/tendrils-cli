use crate::{ActionMode, TendrilBundle};
use crate::filtering::{filter_tendrils, FilterSpec};

fn samples() -> Vec<TendrilBundle> {
    let mut t0 = TendrilBundle::new("SomeApp", vec!["misc0.txt"]);
    t0.link = true;
    t0.profiles = vec![];
    let mut t1 = TendrilBundle::new("SomeApp", vec!["misc1.txt"]);
    t1.link = false;
    t1.profiles = vec!["p1".to_string()];
    let mut t2 = TendrilBundle::new("SomeApp", vec!["misc2.txt"]);
    t2.link = true;
    t2.profiles = vec!["p2".to_string()];
    let mut t3 = TendrilBundle::new("SomeApp", vec!["misc3.txt"]);
    t3.link = false;
    t3.profiles = vec!["p3".to_string()];

    vec![t0, t1, t2, t3]
}

#[test]
fn empty_tendril_list_returns_empty() {
    let tendrils = vec![];
    let filter = FilterSpec {
        mode: None,
        profiles: &[],
    };

    let actual = filter_tendrils(tendrils, filter);

    assert!(actual.is_empty())
}

#[test]
fn mode_filter_is_none_does_not_filter_by_mode() {
    let tendrils = samples();
    let filter = FilterSpec {
        mode: None,
        profiles: &["p1".to_string(), "p2".to_string()],
    };

    let actual = filter_tendrils(tendrils.clone(), filter);

    assert_eq!(
        actual,
        vec![tendrils[0].clone(), tendrils[1].clone(), tendrils[2].clone()]
    );
}

#[test]
fn profile_filter_is_empty_does_not_filter_by_profile() {
    let tendrils = samples();
    let filter = FilterSpec {
        mode: Some(ActionMode::Pull),
        profiles: &[],
    };

    let actual = filter_tendrils(tendrils.clone(), filter);

    assert_eq!(actual, vec![tendrils[1].clone(), tendrils[3].clone()]);
}

#[test]
fn all_filters_are_cumulative() {
    let tendrils = samples();
    let filter = FilterSpec {
        mode: Some(ActionMode::Pull),
        profiles: &["p2".to_string(), "p3".to_string()],
    };

    let actual = filter_tendrils(tendrils.clone(), filter);

    assert_eq!(actual, vec![tendrils[3].clone()]);
}
