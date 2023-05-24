use std::fs;
use std::fs::File;
use std::path::Path;

use rusqlite::{params, Connection, Result, named_params};

fn main() -> Result<()> {
    let db_path = "/home/schelcc/.wallpapers/backgrounds.db";
    let bg_path = "/home/schelcc/.wallpapers/unsorted/";

    // let db = Connection::open(db_path)?;

    refresh_background_db(&db_path, &bg_path)?;

    Ok(())
}


// Refresh backgrounds database, adding entries for any new images
fn refresh_background_db(db_path:&str, bg_path:&str) -> Result<()> {
    let db = Connection::open(db_path)?;

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