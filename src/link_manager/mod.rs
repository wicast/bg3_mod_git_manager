use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Ok, Result};

use iced::{
    alignment,
    widget::{
        button, center, checkbox, column, container, mouse_area, opaque, row, scrollable, stack,
        text, text_editor, text_input,
    },
    window, Color, Element, Task,
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use symlink::symlink_dir;

const PUBLIC_PATH: &str = "Public";
const PROJECTS_PATH: &str = "Projects";
const MODS_PATH: &str = "Mods";
const EDITOR_PATH: &str = "Editor/Mods";

const APP_CONFIG_DIR: &str = "BG3ModGitManager";
const CONFIG_FILE: &str = "Config.json";

#[derive(Debug, Clone)]
pub enum Message {
    ProjectNameInputChanged(String),
    SelectBG3,
    SelectGit,

    ExportAndLink,
    ToggleCreateGit(bool),
    ImportBack,

    ActionPerformed(text_editor::Action),

    HideErr,
}

#[derive(Debug, Default)]
pub struct LinkManager {
    pub project_name: String,
    pub bg3_data_path: PathBuf,
    pub git_root_path: PathBuf,

    create_ignore: bool,

    //Err message
    err_msg: String,
    err_content: text_editor::Content,
    is_perform: bool,
    boot_failed: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    #[serde(default)]
    pub bg3_data_path: String,
}

impl LinkManager {
    /// GUI
    pub fn new() -> (Self, Task<Message>) {
        let try_load_config = || {
            let config_file_path = create_or_get_config_file()?;
            let config_str = fs::read_to_string(&config_file_path)?;
            let config_str = if config_str.is_empty() {
                "{}".to_string()
            } else {
                config_str
            };

            let config: Config = serde_json::from_str(&config_str)?;

            let mgr = LinkManager {
                bg3_data_path: PathBuf::from(config.bg3_data_path),
                create_ignore: true,
                ..Default::default()
            };

            Ok(mgr)
        };
        let mgr = try_load_config().unwrap_or_else(|e| LinkManager {
            err_msg: e.to_string(),
            boot_failed: true,
            ..Default::default()
        });
        (mgr, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let process_msg = || {
            match message {
                Message::ProjectNameInputChanged(name) => {
                    self.project_name = name;
                }
                Message::SelectBG3 => {
                    if let Some(bg3_folder) = FileDialog::new().pick_folder() {
                        Self::check_bg3_data_path(&bg3_folder)?;
                        self.bg3_data_path = bg3_folder;
                        self.save_config()?;
                    }
                }
                Message::SelectGit => {
                    if let Some(git_folder) = FileDialog::new().pick_folder() {
                        self.git_root_path = git_folder;
                        if self.project_name.is_empty() {
                            self.find_project_name()?;
                        }
                    }
                }
                Message::ExportAndLink => {
                    self.export_and_create_soft_link()?;
                    if self.create_ignore {
                        self.create_gitignore()?;
                    }
                }
                Message::ImportBack => {
                    self.import_back()?;
                }
                Message::ToggleCreateGit(b) => {
                    self.create_ignore = b;
                }
                Message::HideErr => {
                    self.err_msg = "".to_string();
                    self.is_perform = false;
                    if self.boot_failed {
                        return Ok(window::get_latest().and_then(window::close));
                    }
                }
                Message::ActionPerformed(action) => {
                    match action {
                        text_editor::Action::Edit(_edit) => {}
                        _ => {
                            self.err_content.perform(action);
                        }
                    }
                    self.is_perform = true;
                }
            };
            Ok(Task::none())
        };

        let res = process_msg();
        self.err_msg = res
            .as_ref()
            .map(|_| "".to_string())
            .unwrap_or_else(|e| e.to_string());
        if !self.err_msg.is_empty() && !self.is_perform {
            self.err_content = text_editor::Content::with_text(&self.err_msg);
        }

        res.unwrap_or(Task::none())
    }

    pub fn view(&self) -> Element<Message> {
        // Elements
        let project_name_text = text("Project Name:");
        let project_name_input = text_input("Not Necessary For Importing", &self.project_name)
            .on_input(Message::ProjectNameInputChanged);

        let bg3_data_path_text = text("BG3 Data Path:");
        let bg3_data_path_input = text_input(
            "",
            self.bg3_data_path.as_os_str().to_str().unwrap_or_default(),
        );
        let select_bg3_data = button("Select Folder").on_press(Message::SelectBG3);

        let git_root_path_text = text("Target Git Path:");
        let git_root_path_input = text_input(
            "",
            self.git_root_path.as_os_str().to_str().unwrap_or_default(),
        );
        let select_git_root = button("Select Folder").on_press(Message::SelectGit);

        let export_to_git_butt = button("Export To Git").on_press(Message::ExportAndLink);
        let export_with_ignore =
            checkbox("Create Ignore Files", self.create_ignore).on_toggle(Message::ToggleCreateGit);
        let export_to_git = row![export_to_git_butt, export_with_ignore].spacing(5);

        let import_back_to_bg3 =
            button("Import Project To BG3 Data Folder").on_press(Message::ImportBack);

        // Layout
        let project_name = row![project_name_text, project_name_input].spacing(5);
        let bg3_data = row![bg3_data_path_text, bg3_data_path_input, select_bg3_data].spacing(5);
        let git_root = row![git_root_path_text, git_root_path_input, select_git_root].spacing(5);

        let content = column![
            project_name,
            bg3_data,
            git_root,
            export_to_git,
            import_back_to_bg3
        ]
        .padding(20)
        .spacing(5);
        if self.err_msg.is_empty() && !self.is_perform {
            content.into()
        } else {
            let alert = container(
                column![
                    text("Error").size(20).color(Color::new(1.0, 0.0, 0.0, 1.0)),
                    text_editor(&self.err_content)
                        .on_action(Message::ActionPerformed)
                        .width(300)
                        .height(100),
                    button("OK").on_press(Message::HideErr)
                ]
                .align_x(alignment::Horizontal::Center)
                .padding(5),
            )
            .style(container::rounded_box);
            modal(content, alert, Message::HideErr)
        }
    }

    fn remove_keep_from_non_empty_dir(dir: &PathBuf) -> Result<bool> {
        let read_dir_iter = fs::read_dir(dir)?;
        for entry in read_dir_iter {
            let ent: fs::DirEntry = entry?;
            if ent.file_type()?.is_dir() {
                continue;
            }
            let name = ent
                .file_name()
                .to_str()
                .ok_or(anyhow!("Convert Project File Name Failed"))?
                .to_string();
            if name != ".gitkeep" {
                std::fs::remove_file(dir.join(".gitkeep"))?;
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Manage soft link
    pub fn export_and_create_soft_link(&self) -> Result<()> {
        //Data
        Self::check_bg3_data_path(&self.bg3_data_path)?;

        let pj_in_data: PathBuf = self.bg3_data_path.join(&self.project_name);
        let to = self.git_root_path.join(&self.project_name);
        self.move_and_link(&pj_in_data, &to)?;
        Self::remove_keep_from_non_empty_dir(&to)?;

        //Public
        let pj_in_public = self
            .bg3_data_path
            .join(PUBLIC_PATH)
            .join(&self.project_name);
        let to = self
            .git_root_path
            .join(PUBLIC_PATH)
            .join(&self.project_name);
        self.move_and_link(&pj_in_public, &to)?;
        Self::remove_keep_from_non_empty_dir(&to)?;

        //Projects
        let pj_in_projects = self
            .bg3_data_path
            .join(PROJECTS_PATH)
            .join(&self.project_name);
        let to = self
            .git_root_path
            .join(PROJECTS_PATH)
            .join(&self.project_name);
        self.move_and_link(&pj_in_projects, &to)?;
        Self::remove_keep_from_non_empty_dir(&to)?;

        //Mods
        let pj_in_mods = self.bg3_data_path.join(MODS_PATH).join(&self.project_name);
        let to = self.git_root_path.join(MODS_PATH).join(&self.project_name);
        self.move_and_link(&pj_in_mods, &to)?;
        Self::remove_keep_from_non_empty_dir(&to)?;

        //Editor
        let pj_in_editor = self
            .bg3_data_path
            .join(EDITOR_PATH)
            .join(&self.project_name);
        let to = self
            .git_root_path
            .join(EDITOR_PATH)
            .join(&self.project_name);
        self.move_and_link(&pj_in_editor, &to)?;
        Self::remove_keep_from_non_empty_dir(&to)?;

        Ok(())
    }

    fn move_and_link(&self, from: &PathBuf, to: &PathBuf) -> Result<()> {
        let move_to = to.join("..");
        if !move_to.exists() {
            fs::create_dir_all(&move_to)?;
        };

        if from.is_dir() && !from.is_symlink() {
            fs_extra::dir::move_dir(from, &move_to, &fs_extra::dir::CopyOptions::new())?;
            symlink_dir(to, from)?;
        } else if !from.exists() && !from.is_symlink() {
            fs::create_dir_all(to)?;
            symlink_dir(to, from)?;
        } else {
            println!("skip exist link {}", from.display());
        }
        let mut gitkeep = fs::File::create(to.join(".gitkeep")).unwrap();
        gitkeep.write_all("".as_bytes())?;
        Ok(())
    }

    fn create_link(&self, from: PathBuf, to: PathBuf) -> Result<()> {
        if !from.exists() {
            bail!("importing failed {} not exists", from.display())
        }

        if to.exists() {
            std::fs::remove_dir(&to)?;
            println!("importing {} already exists, overwrite it", to.display());
        }

        symlink_dir(&from, &to)?;
        Ok(())
    }

    pub fn create_gitignore(&self) -> Result<()> {
        let ignore_str = include_str!("../../template/.gitignore");
        let ignore_path = self.git_root_path.join(".gitignore");
        let mut ignore_file = fs::File::create(ignore_path)?;
        ignore_file.write_all(ignore_str.as_bytes())?;

        let lfs_str = include_str!("../../template/.gitattributes");
        let lfs_path = self.git_root_path.join(".gitattributes");
        let mut lfs_file = fs::File::create(lfs_path)?;
        lfs_file.write_all(lfs_str.as_bytes())?;

        Ok(())
    }

    pub fn import_back(&mut self) -> Result<()> {
        self.find_project_name()?;
        Self::check_bg3_data_path(&self.bg3_data_path)?;

        let to: PathBuf = self.bg3_data_path.join(&self.project_name);
        let from = self.git_root_path.join(&self.project_name);
        self.create_link(from, to)?;

        //Public
        let to = self
            .bg3_data_path
            .join(PUBLIC_PATH)
            .join(&self.project_name);
        let from = self
            .git_root_path
            .join(PUBLIC_PATH)
            .join(&self.project_name);
        self.create_link(from, to)?;

        //Projects
        let to = self
            .bg3_data_path
            .join(PROJECTS_PATH)
            .join(&self.project_name);
        let from = self
            .git_root_path
            .join(PROJECTS_PATH)
            .join(&self.project_name);
        self.create_link(from, to)?;

        //Mods
        let to = self.bg3_data_path.join(MODS_PATH).join(&self.project_name);
        let from = self.git_root_path.join(MODS_PATH).join(&self.project_name);
        self.create_link(from, to)?;

        //Editor
        let to = self
            .bg3_data_path
            .join(EDITOR_PATH)
            .join(&self.project_name);
        let from = self
            .git_root_path
            .join(EDITOR_PATH)
            .join(&self.project_name);
        self.create_link(from, to)?;

        Ok(())
    }

    fn save_config(&self) -> Result<()> {
        let config_file_path = create_or_get_config_file()?;
        let config = Config {
            bg3_data_path: self
                .bg3_data_path
                .as_os_str()
                .to_string_lossy()
                .into_owned(),
        };
        let config_str = serde_json::to_string(&config)?;
        fs::write(config_file_path, config_str)?;

        Ok(())
    }

    fn find_project_name(&mut self) -> Result<()> {
        let proj_path = self.git_root_path.join(PROJECTS_PATH);

        let read_dir_iter = fs::read_dir(proj_path)?;
        for entry in read_dir_iter {
            let ent: fs::DirEntry = entry?;
            self.project_name = ent
                .file_name()
                .to_str()
                .ok_or(anyhow!("Convert Project File Name Failed"))?
                .to_string();
        }
        if self.project_name.is_empty() {
            bail!("No Project Name Found")
        }
        Ok(())
    }

    fn check_bg3_data_path(path: &Path) -> Result<()> {
        let assets = path.join("Assets.pak");
        let gustav = path.join("Gustav.pak");
        let dice_set01 = path.join("DiceSet01.pak");
        if !gustav.exists() || !assets.exists() || !dice_set01.exists() {
            bail!("Not A Valid BG3 Data Path");
        }
        Ok(())
    }
}

fn create_or_get_config_file() -> Result<PathBuf> {
    let base_dir = directories::BaseDirs::new().ok_or(anyhow!("User Base Dir Not Found"))?;
    let config_dir = base_dir.config_dir();
    let app_config_dir = config_dir.join(APP_CONFIG_DIR);
    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir)?;
    }
    let config_file_path = app_config_dir.join(CONFIG_FILE);
    if !config_file_path.exists() {
        fs::File::create(&config_file_path)?;
    }

    Ok(config_file_path)
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.9,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}
