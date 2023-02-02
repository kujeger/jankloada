use anyhow::Result;
use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler};
use iced::{
    executor, theme, Alignment, Application, Color, Command, Element, Length, Settings, Theme,
};
use jankloada_lib::data_manager::DataManager;
use jankloada_lib::mod_data::{ModEntry, ModList, ModProfile};

fn main() -> Result<()> {
    Jankloada::run(Settings::default())?;
    Ok(())
}

#[derive(Debug)]
struct Jankloada {
    data_manager: DataManager,
    mod_list: Option<ModList>,
    profile_list: Vec<String>,
    profile_name: String,
    dirty: bool,
}

#[derive(Debug, Clone)]
enum Message {
    LoadProfile(String),
    SaveProfileAs(String),
    ListProfiles,
    NameProfile(String),
    DeleteProfile(String),
    LoadModList,
    SaveModList,
    ToggleModActive(usize, bool),
}

impl Application for Jankloada {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let data_manager = DataManager::new().expect("Could not initialize!");
        let profile_list = data_manager
            .list_profiles()
            .expect("Failed to read profile dir");
        (
            Self {
                data_manager,
                mod_list: None,
                profile_name: "".to_string(),
                profile_list,
                dirty: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("JANKLOADA")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadProfile(n) => {
                let profile = self
                    .data_manager
                    .load_profile(n.clone())
                    .expect("Failed to load profile!");
                self.mod_list
                    .as_mut()
                    .map(|l| l.apply_profile(profile))
                    .expect("Failed to apply profile!");
                self.profile_name = n;
                self.dirty = true;
            }
            Message::NameProfile(s) => {
                self.profile_name = s;
            }
            Message::SaveProfileAs(n) => {
                let profile = ModProfile::new_from_mod_list(
                    n,
                    &self
                        .mod_list
                        .clone()
                        .expect("Failed to read mod list, even though we already have it?"),
                );
                self.data_manager
                    .save_profile(profile)
                    .expect("Failed to save mod profile!");
                self.reload_profile_list()
                    .expect("Failed to reload profile list");
            }
            Message::DeleteProfile(n) => {
                self.data_manager
                    .delete_profile(n)
                    .expect("Failed to delete profile!");
                self.reload_profile_list()
                    .expect("Failed to reload profile list");
            }
            Message::ListProfiles => {
                self.reload_profile_list()
                    .expect("Failed to reload profile list");
            }
            Message::LoadModList => {
                // TODO: Needs to initialize this safer, warn etc
                let manager = self.data_manager.load_mod_file().unwrap();
                self.mod_list = Some(manager);
                self.profile_name = "".to_string();
                self.dirty = false;
            }
            Message::SaveModList => {
                self.mod_list
                    .as_ref()
                    .map(|ml| self.data_manager.save_to_mod_file(ml.clone()));
                self.dirty = false;
            }
            Message::ToggleModActive(i, b) => {
                self.mod_list
                    .as_mut()
                    .map(|ml| ml.set_mod_active_state(i, b));
                self.dirty = true;
            }
        };
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let load_button = button(if self.mod_list.is_none() {
            "'Ave a look"
        } else {
            "'Ave anuvver look"
        })
        .on_press(Message::LoadModList);
        let mut buttons = row![load_button].spacing(20);
        if self.mod_list.is_some() {
            let text = text("Make it like dis now");
            let save_b = button(text)
                .on_press(Message::SaveModList)
                .style(if self.dirty {
                    theme::Button::Destructive
                } else {
                    theme::Button::Secondary
                });
            buttons = buttons.push(save_b);
        };
        let mut contents = column![buttons].padding(20).align_items(Alignment::Start);
        if self.mod_list.is_some() {
            let mod_file_path = self
                .data_manager
                .resolve_mod_file_path()
                .expect("Failed to get mod file path -- even though we already loaded it?")
                .canonicalize()
                .expect("Failed to canonicalize mod file path")
                .display()
                .to_string();
            contents = contents.push(
                text(
                    "Da grots says dey found it ere, shifty buggers:\n".to_string()
                        + &mod_file_path,
                )
                .size(16),
            );
        }

        let main_panel = self.view_main_overview();
        contents = contents.push(main_panel);
        let contents = contents;
        container(contents).into()
    }
}

impl Jankloada {
    fn reload_profile_list(&mut self) -> Result<()> {
        self.profile_list = self.data_manager.list_profiles()?;
        Ok(())
    }

    fn view_main_overview(&self) -> Element<Message> {
        let mod_pane = scrollable(self.view_modlist()).height(Length::Fill);
        let profile_pane = if self.mod_list.is_some() {
            self.view_profiles()
        } else {
            text("").into()
        };
        row![
            column![mod_pane].width(Length::FillPortion(4)),
            column![profile_pane].width(Length::FillPortion(1))
        ]
        .padding(20)
        .spacing(20)
        .into()
    }

    fn view_modlist(&self) -> Element<Message> {
        let all_mods = self.mod_list.as_ref().map(|m| m.mods()).unwrap_or_default();
        let list: Element<_> = column(
            all_mods
                .iter()
                // Hack to only show twwh3
                .filter(|m| m.game == "warhammer3")
                .enumerate()
                .map(|(i, x)| view_mod_entry(i, x))
                .collect::<Vec<_>>(),
        )
        .padding(20)
        .spacing(5)
        .width(Length::Fill)
        .align_items(Alignment::End)
        .into();
        list
    }

    fn view_profiles(&self) -> Element<Message> {
        let save_current_button = if self.profile_name.is_empty() {
            button("SAVE DIS")
        } else {
            button("SAVE DIS").on_press(Message::SaveProfileAs(self.profile_name.clone()))
        }
        .width(Length::Fill);
        let load_profiles_button = button("WOT")
            .on_press(Message::ListProfiles)
            .style(theme::Button::Positive);
        let profile_name_input = text_input(
            "Ya needs ta NAME it!",
            &self.profile_name,
            Message::NameProfile,
        )
        .width(Length::Fill);
        let profile_list_rows = self
            .profile_list
            .iter()
            .map(|n| {
                {
                    let load = button(n.as_str())
                        .on_press(Message::LoadProfile(n.clone()))
                        .width(Length::Fill);
                    let delete = button("KRUMP")
                        .on_press(Message::DeleteProfile(n.clone()))
                        .style(theme::Button::Destructive);
                    row![load, delete]
                }
                .into()
            })
            .collect();
        column![
            row![save_current_button, load_profiles_button],
            profile_name_input,
            column(profile_list_rows).spacing(5)
        ]
        .spacing(20)
        .into()
    }
}

fn view_mod_entry(i: usize, x: &ModEntry) -> Element<Message> {
    let pri = text(i + 1);
    let game = text(format!("({})", &x.game));
    let exists = x.file_exists();
    let active =
        toggler(None, x.active, move |b| Message::ToggleModActive(i, b)).width(Length::Shrink);
    let name = text(&x.name).width(Length::Fill).style(if exists {
        theme::Text::Default
    } else {
        theme::Text::Color(Color::from_rgb8(255, 165, 0))
    });
    row![pri, active, name, game].spacing(20).into()
}
