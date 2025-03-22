#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_dyn_templates;
#[macro_use] extern crate serde;

use std::env::args;
use std::ffi::OsString;
use std::path::{PathBuf};
use rocket::fs::NamedFile;
use rocket_dyn_templates::Template;
use serde::Serialize;

struct AlbumPath(PathBuf);

#[derive(serde::Serialize)]
struct IndexTemplateContext {
    name: String,
    albums: Vec<String>,
}

#[get("/")]
fn index(albums_path: &rocket::State<AlbumPath>) -> Template {
    let albums: Vec<String> = albums_path.0.read_dir()
        .expect("works")
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.file_type().ok()?.is_dir() {
                    e.file_name().into_string().ok()
                } else {
                    None
                }
            })
        })
        .collect();
    let context = IndexTemplateContext {
        name: "world".to_string(),
        albums,
    };
    Template::render("index", &context)
}

#[get("/static/<album>/<photo>")]
async fn photo(album: &str, photo: &str, albums_path: &rocket::State<AlbumPath>) -> Option<NamedFile> {
    let path = albums_path.0.join(album).join(photo);

    if !path.exists() {
        return None;
    }

    NamedFile::open(path).await.ok()
}

#[launch]
fn rocket() -> _ {
    let dir = args().nth(1).expect("no directory given");
    let path = AlbumPath(PathBuf::from(dir));
    // list directories in path
    for entry in path.0.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            println!("{:?}", entry.path());
        }
    }

    rocket::build()
        .attach(Template::fairing())
        .manage(path)
        .mount("/", routes![photo, index])
}