use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use directories::BaseDirs;

use crate::mod_data::{ModFileDTO, ModList, ModProfile};

const CA_MOD_FILE: &str = "20190104-moddata.dat";

#[derive(Debug)]
pub struct DataManager {
    base_dirs: BaseDirs,
    data_dir: PathBuf,
    custom_mod_file_path: Option<PathBuf>,
}

impl DataManager {
    pub fn new() -> Result<Self> {
        let base_dirs = BaseDirs::new().context("Could not get user base dirs")?;
        let data_dir = base_dirs.data_dir().join("jankloada");
        fs::create_dir_all(base_dirs.data_dir().join("jankloada"))
            .context("Could not create app data dir")?;
        Ok(Self {
            base_dirs,
            data_dir,
            custom_mod_file_path: None,
        })
    }

    #[cfg(target_os = "linux")]
    fn resolve_mod_file_path_platform(&self) -> Result<PathBuf> {
        // magic twwh3 steam id: 1142710
        let steam_proton_append = "steamapps/compatdata/1142710/pfx/drive_c/users/steamuser/AppData/Roaming/The Creative Assembly/Launcher/";
        let paths = vec![
            PathBuf::from(CA_MOD_FILE),
            self.base_dirs
                .home_dir()
                .join(".steam/steam/")
                .join(steam_proton_append)
                .join(CA_MOD_FILE),
            self.base_dirs
                .home_dir()
                .join("Games/SteamLibrary/Default/")
                .join(steam_proton_append)
                .join(CA_MOD_FILE),
        ];
        paths
            .into_iter()
            .find(|p| p.exists())
            .ok_or_else(|| anyhow!("Could not find mod file!"))
    }

    #[cfg(target_os = "windows")]
    fn resolve_mod_file_path_platform(&self) -> Result<PathBuf> {
        let paths = vec![
            PathBuf::from(CA_MOD_FILE),
            self.base_dirs
                .data_dir()
                .join("The Creative Assembly")
                .join("Launcher")
                .join(CA_MOD_FILE),
        ];
        paths
            .into_iter()
            .find(|p| p.exists())
            .ok_or_else(|| anyhow!("Could not find mod file!"))
    }

    #[cfg(target_os = "macos")]
    fn resolve_mod_file_path_platform(&self) -> Result<PathBuf> {
        let paths = vec![PathBuf::from(CA_MOD_FILE)];
        paths
            .into_iter()
            .find(|p| p.exists())
            .ok_or_else(|| anyhow!("Could not find mod file!"))
    }

    pub fn resolve_mod_file_path(&self) -> Result<PathBuf> {
        if let Some(c_path) = self.custom_mod_file_path.clone() {
            Ok(c_path)
        } else {
            self.resolve_mod_file_path_platform()
        }
    }

    fn resolve_profile_path(&self, name: &String) -> PathBuf {
        let mut file_name = self.data_dir.join(name);
        file_name.set_extension("toml");
        file_name
    }

    pub fn load_mod_file(&self) -> Result<ModList> {
        let mod_file_path = self.resolve_mod_file_path()?;
        let data = fs::read_to_string(&mod_file_path)
            .context(format!("Failed to load mod file: {mod_file_path:?}"))?;
        let parsed: ModList = serde_json::from_str::<ModFileDTO>(&data)
            .context("Could not parse mod file contents")?
            .into();
        Ok(parsed)
    }

    pub fn save_to_mod_file(&self, mod_list: ModList) -> Result<()> {
        let mod_file_dto: ModFileDTO = mod_list.into();
        let path = self.resolve_mod_file_path()?;
        let contents = serde_json::to_string_pretty(&mod_file_dto)?;
        fs::write(path, contents).context("Failed to write mod file")?;
        Ok(())
    }

    pub fn save_profile(&self, mod_profile: ModProfile) -> Result<()> {
        let path = self.resolve_profile_path(&mod_profile.name);
        let contents = toml::to_string_pretty(&mod_profile)?;
        fs::write(path, contents).context("Failed to write mod profile")?;
        Ok(())
    }

    pub fn load_profile(&self, name: String) -> Result<ModProfile> {
        let path = self.resolve_profile_path(&name);
        let data = fs::read_to_string(path).context("Could not read mod profile")?;
        let parsed: ModProfile = toml::from_str(&data)?;
        Ok(parsed)
    }

    pub fn delete_profile(&self, name: String) -> Result<()> {
        let path = self.resolve_profile_path(&name);
        fs::remove_file(path)?;
        Ok(())
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        let paths = fs::read_dir(&self.data_dir)
            .context("Failed to read data dir")?
            .filter_map(|i| i.ok())
            .filter(|p| p.path().extension().map(|e| e == "toml").unwrap_or(false))
            .map(|f| f.path())
            .collect::<Vec<_>>();
        let profiles = paths
            .iter()
            .filter_map(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string().replace(".toml", ""))
            .collect::<Vec<_>>();
        Ok(profiles)
    }
}
