use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use symlink::symlink_dir;

const PUBLIC_PATH: &str = "Public";
const PROJETS_PATH: &str = "Projects";
const MODS_PATH: &str = "Mods";
const EDITOR_PATH: &str = "Editor/Mods";

#[derive(Debug)]
pub struct LinkManager {
    pub project_name: String,
    pub bg3_data_path: PathBuf,
    pub git_root_path: PathBuf,
}

impl LinkManager {
    pub fn create_symbol_link_for(&self) -> Result<(), String> {
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
            .join(PROJETS_PATH)
            .join(&self.project_name);
        let to = self
            .git_root_path
            .join(PROJETS_PATH)
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
        if to.exists() {
            let err_s = format!("importing failed {} already exists", to.display());
            return Err(err_s);
        }

        if !from.exists() {
            let err_s = format!("importing failed {} not exists", from.display());
            return Err(err_s);
        }

        symlink_dir(&from, &to).unwrap();

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

    pub fn import_back(&mut self) -> Result<(), String> {
        let proj_path = self.git_root_path
        .join(PROJETS_PATH);

        for entry in fs::read_dir(proj_path).unwrap() {
            let ent = entry.unwrap();
            self.project_name = ent.file_name().into_string().unwrap();
        }

        // println!("{:?}", self);


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
        self.move_and_link(to, from).unwrap();

        //Projects
        let to = self
            .bg3_data_path
            .join(PROJETS_PATH)
            .join(&self.project_name);
        let from = self
            .git_root_path
            .join(PROJETS_PATH)
            .join(&self.project_name);
        self.move_and_link(to, from).unwrap();

        //Mods
        let to = self.bg3_data_path.join(MODS_PATH).join(&self.project_name);
        let from = self.git_root_path.join(MODS_PATH).join(&self.project_name);
        self.move_and_link(to, from).unwrap();

        //Editor
        let to = self
            .bg3_data_path
            .join(EDITOR_PATH)
            .join(&self.project_name);
        let from = self
            .git_root_path
            .join(EDITOR_PATH)
            .join(&self.project_name);
        self.move_and_link(to, from).unwrap();

        Ok(())
    }
}
