pub mod link_manager;

use link_manager::LinkManager;

fn main() -> iced::Result {
    iced::application(
        "Manage BG3 Mod Project With Git",
        LinkManager::update,
        LinkManager::view,
    )
    .run_with(LinkManager::new)
}
