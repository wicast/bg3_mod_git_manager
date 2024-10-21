pub mod link_manager;
use std::path::PathBuf;

use link_manager::LinkManager;

use argh::FromArgs;
use rfd::FileDialog;

#[derive(FromArgs, Default)]
/// CLI Args
struct Args {
    ///project name
    #[argh(option, short = 'n')]
    project_name: Option<String>,

    ///the BG3 Data Path
    #[argh(option, short = 'b')]
    bg3_data_root: String,

    ///git root
    #[argh(option, short = 'g')]
    git_root: String,

    ///improt to BG3 toolkit
    #[argh(switch, short = 'i')]
    is_import: bool,
}

fn main() -> iced::Result {
    // let args: Args = Args::default();

    // let mut lk = LinkManager::new(
    //     &args.project_name.unwrap_or_default(),
    //     FileDialog::new().pick_folder().unwrap().to_str().unwrap(),
    //     FileDialog::new().pick_folder().unwrap().to_str().unwrap(),
    // );

    // if !args.is_import {
    //     lk.create_symbol_link_for().unwrap();
    //     lk.create_gitignore();
    // } else {
    //     lk.import_back().unwrap();
    // }

    iced::run("Manage BG3 Mod Project With Git", LinkManager::update, LinkManager::view)
}
