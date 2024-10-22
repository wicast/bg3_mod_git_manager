use std::{fs, io::Write, path::PathBuf};

use iced::{widget::{button, column, row, text, text_input, Column}, Task};
use rfd::FileDialog;
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
    ImportBack,
}

#[derive(Debug, Default)]
pub struct LinkManager {
    pub project_name: String,
    pub bg3_data_path: PathBuf,
    pub git_root_path: PathBuf,
}

struct Config {
    pub bg3_data_path: String
}

impl LinkManager {
    /// GUI
    pub fn new() -> (Self, Task<Message>)  {
        let base_dir = directories::BaseDirs::new().unwrap();
        let config_dir = base_dir.config_dir();
        let app_config_dir = config_dir.join(APP_CONFIG_DIR);
        if !app_config_dir.exists() {
            fs::create_dir_all(&app_config_dir).unwrap();
        }
        let coinfig_file_path = app_config_dir.join(CONFIG_FILE);
        let mut mgr = LinkManager::default();
        mgr.bg3_data_path = PathBuf::from("todo");
        (mgr , Task::none())
    }
    
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ProjectNameInputChanged(name) => {
                self.project_name = name;
            }
            Message::SelectBG3 => {
                let bg3_folder = FileDialog::new().pick_folder().unwrap_or_default();
                //TODO check bg3 data path
                self.bg3_data_path = bg3_folder;
                //TODO save config
            }
            Message::SelectGit => {
                let git_folder = FileDialog::new().pick_folder().unwrap_or_default();
                self.git_root_path = git_folder;
            }
            Message::ExportAndLink => {
                self.export_and_create_symbol_link().unwrap();
                self.create_gitignore();
            }
            Message::ImportBack => {
                self.import_back().unwrap();
            }
        }
    }

    pub fn view(&self) -> Column<Message> {
        // Elements
        let project_name_text = text("Project Name:");
        let project_name_input =
            text_input("Optional For Import", &self.project_name).on_input(Message::ProjectNameInputChanged);

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

        let export_to_git = button("Export To Git").on_press(Message::ExportAndLink);

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

    pub fn export_and_create_symbol_link(&self) -> Result<(), String> {
        //Data
        if !self.bg3_data_path.exists() {
            let mut s = String::from(self.bg3_data_path.to_str().unwrap());
            s.push_str(", BG3 Data Path Not found");
            return Err(s);
        }

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
            // println!("{}", from.display());
            // println!("{}", move_to.display());
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
            // let err_s = format!("importing failed {} already exists", to.display());
            // return Err(err_s);
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
        self.find_project_name();

        if !self.bg3_data_path.exists() {
            let mut s = String::from(self.bg3_data_path.to_str().unwrap());
            s.push_str(", BG3 Data Path Not found");
            return Err(s);
        }

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

    fn find_project_name(&mut self) {
        let proj_path = self.git_root_path.join(PROJECTS_PATH);

        for entry in fs::read_dir(proj_path).unwrap() {
            let ent = entry.unwrap();
            self.project_name = ent.file_name().into_string().unwrap();
        }
    }
}
