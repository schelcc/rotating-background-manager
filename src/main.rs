// mod img_tools;

use std::fs;
use std::fs::File;
use std::path::Path;

use rand::seq::SliceRandom;
use rusqlite::{params, Connection, Result, named_params};

fn main() -> Result<()> {
    let db_path = "/home/schelcc/.wallpapers/backgrounds.db";
    let bg_path = "/home/schelcc/.wallpapers/unsorted/";

    let db = Connection::open(db_path)?;

    refresh_background_db(&db, &bg_path)?;

    // match select_true_random(&db) {
    //     Err(_) => (),
    //     Ok(res) => println!("Selected: {}", res)
    // };

    Ok(())
}

fn select_true_random(db:&Connection) -> Result<String, ()> {
    let mut stmt = match db.prepare("SELECT backgrounds.path FROM backgrounds") {
        Err(why) => {
            println!("[ERR] Selection statement preparation: {:?}", why);
            return Err(())
        },
        Ok(res) => res
    };

    let results = match stmt.query_map([], |r| r.get::<usize, String>(0)) {
        Err(why) => {
            println!("[ERR] Query map: {:?}", why);
            return Err(())
        },
        Ok(res) => res
    };

    let mut selection_vec : Vec<String> = Vec::new();

    for res in results {
        selection_vec.push(match res {
            Err(why) => {
                println!("[ERR] Vector construction: {:?}", why);
                return Err(())
            },
            Ok(res) => res
        });
    };

    let selection : String = match selection_vec.choose(&mut rand::thread_rng()) {
        None => {
            println!("[ERR] Random selection: No result from random selection");
            return Err(())
        },
        Some(sel) => sel.to_string()
    };

    Ok(selection)
}


// Refresh backgrounds database, adding entries for any new images
fn refresh_background_db(db:&Connection, bg_path:&str) -> Result<()> {
    // Create auxiliary table for matching
    db.execute("CREATE TABLE auxiliary (path TEXT)", ())?;

    let insertion = "INSERT INTO auxiliary VALUES (?1)";
    let mut auxiliary_insertion_stmt = db.prepare(insertion)?;

    let mut insertion_count : u64 = 0;

    // Read bg_path for all files (TODO: Glob for just image types)
    match fs::read_dir(bg_path) {
        Err(why) => println!("Error: {:?}", why.kind()),
        Ok(paths) => for path in paths {
            auxiliary_insertion_stmt.execute( 
                    params![
                        path.as_ref()
                        .unwrap()
                        .path()
                        .display()
                        .to_string()])?;
        }
    }

    // Query all rows in aux and not in backgrounds, then update backgrounds accordingly
    let query = "SELECT auxiliary.path FROM auxiliary WHERE auxiliary.path NOT IN (SELECT backgrounds.path FROM backgrounds)";
    let mut query_stmt = db.prepare(query)?;

    // Prepare an insertion statement for query results
    let insertion = "INSERT INTO backgrounds VALUES (?1, ?2)";
    let mut insertion_stmt = db.prepare(insertion)?;

    // Retrieve the path entry
    let results = query_stmt.query_map([], |r| r.get::<usize, String>(0))?;

    // We specifically loop rather than map here because for the compiler to actually execute the query,
    // the resulting iterator must be consumed
    for item in results {
        insertion_stmt.execute(params![item?, 0u64])?;
        insertion_count += 1;
    };

    // Drop the auxiliary table
    db.execute("DROP TABLE auxiliary", ())?;

    println!("[INFO] Backgrounds database refreshed - {} new entries added", insertion_count);

    // Return the rusqlite Ok result if we get here
    Ok(())
}