use std::{
    fs, io::Write, path::{Path, PathBuf}
};

use symlink::symlink_dir;

const PUBLIC_PATH: &str = "Public";
const PROJETS_PATH: &str = "Projects";
const MODS_PATH: &str = "Mods";
const EDITOR_PATH: &str = "Editor/Mods";

pub struct LinkManager {
    pub bg3_data_path: PathBuf,
    pub git_root_path: PathBuf,
}

impl LinkManager {
    pub fn create_symbol_link_for(&self, project_name: &str) -> Result<(), String> {
        //Data
        if !self.bg3_data_path.exists() {
            let mut s = String::from(self.bg3_data_path.to_str().unwrap());
            s.push_str(", BG3 Data Path Not found");
            return Err(s);
        }

        let pj_in_data: PathBuf = self.bg3_data_path.join(project_name);
        let to = self.git_root_path.join(project_name);
        self.move_and_link(pj_in_data, to).unwrap();

        //Public
        let pj_in_public = self.bg3_data_path.join(PUBLIC_PATH).join(project_name);
        let to = self.git_root_path.join(PUBLIC_PATH).join(project_name);
        self.move_and_link(pj_in_public, to).unwrap();

        //Projects
        let pj_in_projects = self.bg3_data_path.join(PROJETS_PATH).join(project_name);
        let to = self.git_root_path.join(PROJETS_PATH).join(project_name);
        self.move_and_link(pj_in_projects, to).unwrap();

        //Mods
        let pj_in_mods = self.bg3_data_path.join(MODS_PATH).join(project_name);
        let to = self.git_root_path.join(MODS_PATH).join(project_name);
        self.move_and_link(pj_in_mods, to).unwrap();

        //Editor
        let pj_in_editor =  self.bg3_data_path.join(EDITOR_PATH).join(project_name);
        let to = self.git_root_path.join(EDITOR_PATH).join(project_name);
        self.move_and_link(pj_in_editor, to).unwrap();

        Ok(())
    }

    fn move_and_link(&self, from: PathBuf, to: PathBuf) -> Result<(), String> {
        let move_to = to.join("..");
        if !move_to.exists() {
            fs::create_dir_all(&move_to).unwrap();
        };

        if from.is_dir() && !from.is_symlink() {
            println!("{}", from.display());
            println!("{}", move_to.display());
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

    pub fn create_gitignore(&self) {
        let ignore_str = "Mods/*/Story/**/*.*
!Mods/*/Story/RawFiles/Goals/*
";
        let ignore_path = self.git_root_path.join(".gitignore");
        let mut ignore_file = fs::File::create(ignore_path).unwrap();
        ignore_file.write_all(ignore_str.as_bytes()).unwrap();
    }
}
