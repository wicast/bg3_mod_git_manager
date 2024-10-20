pub mod link_manager;
use std::path::{Path, PathBuf};

use link_manager::LinkManager;

use argh::FromArgs;

#[derive(FromArgs)]
/// CLI Args
struct Args {
    ///project name
    #[argh(option, short='n')]
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

fn main() {
    let args: Args = argh::from_env();
    let mut lk = LinkManager {
        project_name: args.project_name.unwrap_or_default(),
        bg3_data_path: PathBuf::from(&args.bg3_data_root),
        git_root_path: PathBuf::from(&args.git_root),
    };

    if !args.is_import {
        lk.create_symbol_link_for().unwrap();
        lk.create_gitignore();
    } else {
        lk.import_back().unwrap();
    }
}
