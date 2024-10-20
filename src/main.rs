pub mod link_manager;
use std::path::{Path, PathBuf};

use link_manager::LinkManager;

use argh::FromArgs;

#[derive(FromArgs)]
/// CLI Args
struct Args {
    ///project name
    #[argh(positional)]
    project_name: String,

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
    let lk = LinkManager {
        bg3_data_path: PathBuf::from(&args.bg3_data_root),
        git_root_path: PathBuf::from(&args.git_root),
    };

    if !args.is_import {
        lk.create_symbol_link_for(&args.project_name).unwrap();
        lk.create_gitignore();
    }
}