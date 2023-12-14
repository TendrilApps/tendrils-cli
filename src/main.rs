use tendrils::{
    get_tendril_overrides,
    get_tendrils,
    get_tendrils_folder,
    resolve_overrides,
};

fn main() {
    let tendrils_folder = get_tendrils_folder(&std::env::current_dir()
        .expect("Could not get the current directory"))
        .expect("Could not find a Tendrils folder");

    let common_tendrils = get_tendrils(&tendrils_folder)
        .expect("Could not import the tendrils.json file");

    let override_tendrils = get_tendril_overrides(&tendrils_folder)
        .expect("Could not import the tendrils-overrides.json file");

    if override_tendrils.is_empty() {
        println!("No local overrides were found.")
    }

    let resolved_tendrils =
        resolve_overrides(&common_tendrils, &override_tendrils);

    print!("{:#?}", resolved_tendrils);
}
