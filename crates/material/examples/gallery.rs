//! Desktop entrypoint for the shared Material gallery.
#[path = "gallery/mod.rs"]
mod gallery;

fn main() {
    gallery::run(gpui_platform::application(), |_| {});
}
