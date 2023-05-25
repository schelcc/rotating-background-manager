// mod img_tools;

use std::fs;

use rusqlite::{params, Connection, Result};

enum RandomState {
    FullShuffle,
    MinUses
}

fn main() -> Result<()> {
    let db_path = "/home/schelcc/.wallpapers/backgrounds.db";
    let bg_path = "/home/schelcc/.wallpapers/unsorted/";

    let db = Connection::open(&db_path)?;

    // For now, setting up to be used as one-time execution
    refresh_background_db(&db, &bg_path)?;

    select_set_update(&db, RandomState::MinUses);

    Ok(())
}

fn select_set_update(db:&Connection, rand:RandomState) -> () {
    let path = match select_random(&db, rand) {
        Err(_) => String::from("/home/schelcc/.wallpapers/default_bg.jpg"),
        Ok(path) => path
    };

    wallpaper::set_from_path(&path).unwrap();

    match db.execute("UPDATE backgrounds SET uses = uses + 1 WHERE path = ?1", params![&path]) {
        Err(why) => println!("[ERR] Update error: {:?}", why),
        Ok(_) => ()
    }
}

fn select_random(db:&Connection, rand:RandomState) -> Result<String, rusqlite::Error> {
    let mut stmt = match &rand {
        RandomState::FullShuffle => {
            db.prepare("SELECT backgrounds.path FROM backgrounds ORDER BY RANDOM() LIMIT 1")
                .expect("[ERR] Random selection err")
        },
        RandomState::MinUses => {
            db.prepare("SELECT backgrounds.path FROM backgrounds 
                            WHERE backgrounds.uses = (SELECT min(backgrounds.uses) 
                            FROM backgrounds) ORDER BY RANDOM() LIMIT 1")
                .expect("[ERR] Random selection err")
        }
    };

    let mut results = stmt.query_map([], |r| r.get::<usize, String>(0))?;

    results.nth(0).unwrap()
}


// Refresh backgrounds database, adding entries for any new images
fn refresh_background_db(db:&Connection, bg_path:&str) -> Result<()> {

    let mut insertion_count : u64 = 0;

    let mut update = db.prepare("INSERT INTO backgrounds VALUES (?1, 0)")?;

    // Read bg_path for all files (TODO: Glob for just image types)
    match fs::read_dir(bg_path) {
        Err(why) => println!("Error: {:?}", why.kind()),
        Ok(paths) => for path in paths {
            let path_str = path.as_ref()
            .unwrap()
            .path()
            .display()
            .to_string();

            match update.execute(params![path_str]) {
                Err(_) => (),
                Ok(_) => {insertion_count += 1;}
            };
        }
    }

    println!("[INFO] Backgrounds database refreshed - {} new entries added", insertion_count);

    // Return the rusqlite Ok result if we get here
    Ok(())
}