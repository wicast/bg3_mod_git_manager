use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use iced::{
    widget::{button, checkbox, column, row, text, text_input, Column},
    Task,
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
}

#[derive(Debug, Default)]
pub struct LinkManager {
    pub project_name: String,
    pub bg3_data_path: PathBuf,
    pub git_root_path: PathBuf,

    create_ignore: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    #[serde(default)]
    pub bg3_data_path: String,
}

impl LinkManager {
    /// GUI
    pub fn new() -> (Self, Task<Message>) {
        let config_file_path = create_or_get_config_file();
        let config_str = fs::read_to_string(&config_file_path).unwrap();
        let config_str = if config_str.is_empty() {
            "{}".to_string()
        } else {
            config_str
        };

        let config: Config = serde_json::from_str(&config_str).unwrap();

        let mgr = LinkManager {
            bg3_data_path: PathBuf::from(config.bg3_data_path),
            create_ignore: true,
            ..Default::default()
        };
        (mgr, Task::none())
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ProjectNameInputChanged(name) => {
                self.project_name = name;
            }
            Message::SelectBG3 => {
                let bg3_folder = FileDialog::new().pick_folder().unwrap_or_default();

                Self::check_bg3_data_path(&bg3_folder).unwrap();
                self.bg3_data_path = bg3_folder;
                self.save_config();
            }
            Message::SelectGit => {
                let git_folder = FileDialog::new().pick_folder().unwrap_or_default();
                self.git_root_path = git_folder;
                self.find_project_name().unwrap_or_default();
            }
            Message::ExportAndLink => {
                self.export_and_create_soft_link().unwrap();
                if self.create_ignore {
                    self.create_gitignore();
                }
            }
            Message::ImportBack => {
                self.import_back().unwrap();
            }
            Message::ToggleCreateGit(b) => {
                self.create_ignore = b;
            }
        }
    }

    pub fn view(&self) -> Column<Message> {
        // Elements
        let project_name_text = text("Project Name:");
        let project_name_input = text_input("Optional For Importing", &self.project_name)
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

        column![
            project_name,
            bg3_data,
            git_root,
            export_to_git,
            import_back_to_bg3
        ]
        .padding(20)
        .spacing(5)
    }

    /// Manage soft link
    pub fn export_and_create_soft_link(&self) -> Result<(), String> {
        //Data
        Self::check_bg3_data_path(&self.bg3_data_path).unwrap();

        let pj_in_data: PathBuf = self.bg3_data_path.join(&self.project_name);
        let to = self.git_root_path.join(&self.project_name);
        self.move_and_link(pj_in_data, to).unwrap();

        //Public
        let pj_in_public = self
            .bg3_data_path
            .join(PUBLIC_PATH)
            .join(&self.project_name);
        let to = self
            .git_root_path
            .join(PUBLIC_PATH)
            .join(&self.project_name);
        self.move_and_link(pj_in_public, to).unwrap();

        //Projects
        let pj_in_projects = self
            .bg3_data_path
            .join(PROJECTS_PATH)
            .join(&self.project_name);
        let to = self
            .git_root_path
            .join(PROJECTS_PATH)
            .join(&self.project_name);
        self.move_and_link(pj_in_projects, to).unwrap();

        //Mods
        let pj_in_mods = self.bg3_data_path.join(MODS_PATH).join(&self.project_name);
        let to = self.git_root_path.join(MODS_PATH).join(&self.project_name);
        self.move_and_link(pj_in_mods, to).unwrap();

        //Editor
        let pj_in_editor = self
            .bg3_data_path
            .join(EDITOR_PATH)
            .join(&self.project_name);
        let to = self
            .git_root_path
            .join(EDITOR_PATH)
            .join(&self.project_name);
        self.move_and_link(pj_in_editor, to).unwrap();

        Ok(())
    }

    fn move_and_link(&self, from: PathBuf, to: PathBuf) -> Result<(), String> {
        let move_to = to.join("..");
        if !move_to.exists() {
            fs::create_dir_all(&move_to).unwrap();
        };

        if from.is_dir() && !from.is_symlink() {
            fs_extra::dir::move_dir(&from, &move_to, &fs_extra::dir::CopyOptions::new()).unwrap();
            symlink_dir(&to, &from).unwrap();
        } else if !from.exists() && !from.is_symlink() {
            fs::create_dir_all(&to).unwrap();
            symlink_dir(&to, &from).unwrap();
        } else {
            println!("skip exist link {}", from.display());
        }
        Ok(())
    }

    fn create_link(&self, from: PathBuf, to: PathBuf) -> Result<(), String> {
        if !from.exists() {
            let err_s = format!("importing failed {} not exists", from.display());
            return Err(err_s);
        }

        if to.exists() {
            std::fs::remove_file(&to).unwrap();
            println!("importing {} already exists, overwrite it", to.display());
        }

        symlink_dir(&from, &to).unwrap();
        Ok(())
    }

    pub fn create_gitignore(&self) {
        let ignore_str = include_str!("../../template/.gitignore");
        let ignore_path = self.git_root_path.join(".gitignore");
        let mut ignore_file = fs::File::create(ignore_path).unwrap();
        ignore_file.write_all(ignore_str.as_bytes()).unwrap();

        let lfs_str = include_str!("../../template/.gitattributes");
        let lfs_path = self.git_root_path.join(".gitattributes");
        let mut lfs_file = fs::File::create(lfs_path).unwrap();
        lfs_file.write_all(lfs_str.as_bytes()).unwrap();
    }

    pub fn import_back(&mut self) -> Result<(), String> {
        self.find_project_name().unwrap();
        Self::check_bg3_data_path(&self.bg3_data_path).unwrap();

        let to: PathBuf = self.bg3_data_path.join(&self.project_name);
        let from = self.git_root_path.join(&self.project_name);
        self.create_link(from, to).unwrap();

        //Public
        let to = self
            .bg3_data_path
            .join(PUBLIC_PATH)
            .join(&self.project_name);
        let from = self
            .git_root_path
            .join(PUBLIC_PATH)
            .join(&self.project_name);
        self.create_link(from, to).unwrap();

        //Projects
        let to = self
            .bg3_data_path
            .join(PROJECTS_PATH)
            .join(&self.project_name);
        let from = self
            .git_root_path
            .join(PROJECTS_PATH)
            .join(&self.project_name);
        self.create_link(from, to).unwrap();

        //Mods
        let to = self.bg3_data_path.join(MODS_PATH).join(&self.project_name);
        let from = self.git_root_path.join(MODS_PATH).join(&self.project_name);
        self.create_link(from, to).unwrap();

        //Editor
        let to = self
            .bg3_data_path
            .join(EDITOR_PATH)
            .join(&self.project_name);
        let from = self
            .git_root_path
            .join(EDITOR_PATH)
            .join(&self.project_name);
        self.create_link(from, to).unwrap();

        Ok(())
    }

    fn save_config(&self) {
        let config_file_path = create_or_get_config_file();
        let config = Config {
            bg3_data_path: self
                .bg3_data_path
                .as_os_str()
                .to_string_lossy()
                .into_owned(),
        };
        let config_str = serde_json::to_string(&config).unwrap();
        fs::write(config_file_path, config_str).unwrap();
    }

    fn find_project_name(&mut self) -> Result<(), String> {
        let proj_path = self.git_root_path.join(PROJECTS_PATH);

        let read_dir_iter = fs::read_dir(proj_path).map_err(|e| e.to_string())?;
        for entry in read_dir_iter {
            let ent: fs::DirEntry = entry.map_err(|e| e.to_string())?;
            self.project_name = ent.file_name().into_string().unwrap();
        }
        if self.project_name.is_empty() {
            return Err("No Project Name Found".to_string());
        }
        Ok(())
    }

    fn check_bg3_data_path(path: &Path) -> Result<(), String> {
        let assets = path.join("Assets.pak");
        let gustav = path.join("Gustav.pak");
        let dice_set01 = path.join("DiceSet01.pak");
        if !gustav.exists() || !assets.exists() || !dice_set01.exists() {
            return Err("Not A Valid BG3 Data Path".to_string());
        }
        Ok(())
    }
}

fn create_or_get_config_file() -> PathBuf {
    let base_dir = directories::BaseDirs::new().unwrap();
    let config_dir = base_dir.config_dir();
    let app_config_dir = config_dir.join(APP_CONFIG_DIR);
    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir).unwrap();
    }
    let config_file_path = app_config_dir.join(CONFIG_FILE);
    if !config_file_path.exists() {
        fs::File::create(&config_file_path).unwrap();
    }
    config_file_path
}
