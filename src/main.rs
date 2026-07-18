use std::{path::PathBuf, sync::Arc};

use chrono::Local;
use daydream::{
    app::{
        message::{Request, RequestWrapper},
        state::App,
    },
    repr::storage::FsStorage,
};
use iced::Task;

fn main() -> iced::Result {
    env_logger::init();
    let storage = Arc::new(FsStorage::new(PathBuf::from("test")));
    iced::application(
        move || {
            (
                App::new(storage.clone()),
                Task::done(RequestWrapper::new(Request::ShowDay(
                    Local::now().date_naive(),
                ))),
            )
        },
        App::update,
        App::view,
    )
    .run()
}
