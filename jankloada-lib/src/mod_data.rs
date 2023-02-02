use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct ModUUID(pub String);

#[derive(Serialize, Deserialize, Debug)]
pub struct ModFileDTO(pub Vec<ModEntryDTO>);

#[derive(Serialize, Deserialize, Debug)]
pub struct ModEntryDTO {
    uuid: ModUUID,
    name: String,
    active: bool,
    category: String,
    game: String,
    order: usize,
    owned: bool,
    packfile: String,
    short: String,
}

impl From<ModList> for ModFileDTO {
    fn from(modlist: ModList) -> Self {
        Self(
            modlist
                .0
                .into_iter()
                .enumerate()
                .map(|(i, m)| ModEntryDTO {
                    uuid: m.uuid,
                    active: m.active,
                    category: m.category,
                    game: m.game,
                    name: m.name,
                    order: i + 1, // Launcher treats 0 as "last"
                    owned: m.owned,
                    packfile: m.packfile,
                    short: m.short,
                })
                .collect(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModEntry {
    pub uuid: ModUUID,
    pub name: String,
    pub active: bool,
    pub category: String,
    pub game: String,
    pub owned: bool,
    pub packfile: String,
    pub short: String,
}

impl ModEntry {
    pub fn file_exists(&self) -> bool {
        // Wine/Proton workaround hack.
        let path = if self.packfile.starts_with("Z:/") {
            &self.packfile[2..]
        } else {
            self.packfile.as_str()
        };
        Path::new(path).exists()
    }

    fn set_active(&mut self, t: bool) {
        self.active = t
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModList(Vec<ModEntry>);

impl ModList {
    pub fn mods(&self) -> Vec<&ModEntry> {
        self.0.iter().collect()
    }

    pub fn prune_missing(&mut self) {
        // TODO: here we check if any mod files are missing, and if so, remove
        // them from the mod list
        todo!()
    }

    pub fn get_missing(&self) -> Vec<&ModEntry> {
        self.0.iter().filter(|m| !m.file_exists()).collect()
    }

    pub fn get_active(&self) -> Vec<&ModEntry> {
        self.0.iter().filter(|m| m.active).collect()
    }

    pub fn deactivate_all(&mut self) {
        self.0.iter_mut().for_each(|m| m.active = false)
    }

    pub fn apply_profile(&mut self, profile: ModProfile) {
        self.deactivate_all();

        let mut mods: Vec<ModEntry> = Vec::new();
        mods.append(&mut self.0);

        let (mut in_profile, mut outside_profile): (Vec<_>, _) = mods
            .into_iter()
            .partition(|m| profile.active_mods.contains(&m.uuid));
        in_profile.iter_mut().for_each(|mut m| m.active = true);

        let mut in_profile_ordered: Vec<ModEntry> = profile
            .active_mods
            .iter()
            .filter_map(|m| {
                in_profile
                    .iter()
                    .position(|e| &e.uuid == m)
                    .map(|i| in_profile.remove(i))
            })
            .collect();

        self.0.append(&mut in_profile_ordered);
        // Should we somehow fail to order item(s), append here to avoid data loss
        self.0.append(&mut in_profile);
        self.0.append(&mut outside_profile);
    }

    pub fn set_mod_active_state(&mut self, index: usize, b: bool) -> Result<()> {
        self.0
            .get_mut(index)
            .ok_or_else(|| anyhow!("as"))
            .map(|m| m.set_active(b))
    }
}

impl From<ModFileDTO> for ModList {
    fn from(mut dto: ModFileDTO) -> Self {
        dto.0.sort_by_key(|m| m.order);
        Self(
            dto.0
                .into_iter()
                .map(|m| ModEntry {
                    uuid: m.uuid,
                    active: m.active,
                    category: m.category,
                    game: m.game,
                    name: m.name,
                    owned: m.owned,
                    packfile: m.packfile,
                    short: m.short,
                })
                .collect(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModProfile {
    pub name: String,
    pub active_mods: Vec<ModUUID>,
}

impl ModProfile {
    pub fn new_from_mod_list(name: String, mod_list: &ModList) -> Self {
        Self {
            name,
            active_mods: mod_list
                .get_active()
                .iter()
                .map(|m| m.uuid.clone())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mod_data::{ModEntry, ModList, ModProfile, ModUUID};

    #[test]
    fn applying_profile_works() {
        let mut mod_list = ModList(vec![
            ModEntry {
                uuid: ModUUID("one".to_string()),
                name: "One".to_string(),
                active: false,
                category: "foo".to_string(),
                game: "foo".to_string(),
                owned: true,
                packfile: "/foo.pack".to_string(),
                short: "the foo mod".to_string(),
            },
            ModEntry {
                uuid: ModUUID("two".to_string()),
                name: "Two".to_string(),
                active: true,
                category: "foo".to_string(),
                game: "foo".to_string(),
                owned: true,
                packfile: "/foo.pack".to_string(),
                short: "the foo mod".to_string(),
            },
        ]);
        let mod_profile = ModProfile {
            name: "some_profile".to_string(),
            active_mods: vec![ModUUID("one".to_string())],
        };
        mod_list.apply_profile(mod_profile);

        assert!(mod_list.0[0].active);
        assert_eq!("One".to_string(), mod_list.0[0].name);
        assert!(!mod_list.0[1].active);
        assert_eq!(2, mod_list.0.len())
    }
}
