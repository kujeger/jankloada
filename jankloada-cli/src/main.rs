use anyhow::Result;
use jankloada_lib::{
    data_manager::DataManager,
    mod_data::{ModFileDTO, ModProfile},
};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let arg_cmd = args.get(1);
    let arg_profile = args.get(2);

    let data_manager = DataManager::new()?;

    let mod_list = data_manager.load_mod_file()?;

    if let Some(cmd) = arg_cmd {
        match cmd.as_str() {
            "print" => println!(
                "{}",
                serde_json::to_string_pretty::<ModFileDTO>(&mod_list.into())?
            ),
            "current" => {
                let active_mods = mod_list.get_active();
                for (i, n) in active_mods.iter().enumerate() {
                    println!("{i} - {}", n.name)
                }
            }
            "missing" => {
                let missing_mods = mod_list.get_missing();
                for (i, n) in missing_mods.iter().enumerate() {
                    println!("{i} - {}", n.name)
                }
            }
            "save" => {
                if let Some(name) = arg_profile {
                    let mod_profile = ModProfile::new_from_mod_list(name.to_owned(), &mod_list);
                    data_manager.save_profile(mod_profile)?;
                    println!("Profile {name} saved.")
                } else {
                    println!("Missing profile name")
                }
            }
            "list" => {
                let profiles = data_manager.list_profiles()?;
                for item in profiles {
                    println!("{item}")
                }
            }
            "show" => {
                if let Some(name) = arg_profile {
                    let profile = data_manager.load_profile(name.to_owned())?;
                    println!("Profile \"{}\"", profile.name);
                    for (i, n) in profile.active_mods.iter().enumerate() {
                        println!("{i} - {}", n.0)
                    }
                } else {
                    println!("Missing profile name")
                }
            }
            "apply" => {
                if let Some(name) = arg_profile {
                    let profile = data_manager.load_profile(name.to_owned())?;
                    let mut mod_list = mod_list;
                    mod_list.apply_profile(profile);

                    data_manager.save_to_mod_file(mod_list)?;
                } else {
                    println!("Missing profile name")
                }
            }
            other => println!("Unkown command: {other}"),
        }
    } else {
        println!("Missing command")
    }
    Ok(())
}
