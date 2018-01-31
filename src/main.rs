#[macro_use]
extern crate clap;
extern crate meval;
extern crate itertools;
extern crate sqlite;

use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::fs::File;
use itertools::Itertools;
use sqlite::Value;

#[cfg(feature = "yaml")]
fn main()
{
    use clap::App;

    let yml = load_yaml!("cli.yaml");
    let m = App::from_yaml(yml).get_matches();

    match m.subcommand_name()
    {
        Some("generate") => generate(m),
        Some("lookup") => lookup(m),
        None => println!("show help here. TODO: me!"),
           _ => println!("Some other subcommand was used"),
    }
}

#[cfg(feature = "yaml")]
fn generate(m_opts: clap::ArgMatches)
{
    if let Some(ref m_opts) = m_opts.subcommand_matches("generate")
    {
        let lib = m_opts.value_of("lib").unwrap();
        let gb_ratios = m_opts.value_of("gb_ratios").unwrap();
        let m: f64 = m_opts.value_of("gear_m").unwrap().parse::<f64>().unwrap();  // TODO dafuq?
        let n: f64 = m_opts.value_of("gear_n").unwrap().parse::<f64>().unwrap();  // TODO same
        let pitch: f64 = m_opts.value_of("pitch").unwrap().parse::<f64>().unwrap(); // TODO and me
        let mnp: f64 = (m/n)*pitch;

        // db output file string
        let db_str = m_opts.value_of("out").unwrap();

        let db_journal_mode = "PRAGMA JOURNAL_MODE = OFF;"; // disable journal to increase write performance
        let db_synchronous = "PRAGMA SYNCHRONOUS = OFF;";   // disable rollback sync
        let db_exclusive = "PRAGMA LOCKING_MODE = EXCLUSIVE;"; // exclusive for more fasters
        let db_tempstore = "PRAGMA TEMP_STORE = MEMORY;"; // temp objects in RAM
        let db_cache_size = "PRAGMA PAGE_COUNT = 10000;"; // TODO: tune me
        let db_page_size = "PRAGMA PAGE_SIZE = 16384;";   // TODO: tune me
        let db_drop_table = "DROP TABLE IF EXISTS `pitches`;";
        let db_create_table = "
            CREATE TABLE `pitches` (
                `id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,
                `pitch` REAL NOT NULL,
                `m` INTEGER NOT NULL,
                `n` INTEGER NOT NULL,
                `a` INTEGER NOT NULL,
                `b` INTEGER NOT NULL,
                `c` INTEGER NOT NULL,
                `d` INTEGER NOT NULL,
                `r` REAL NOT NULL);";
        let db_create_idx = "
            CREATE INDEX `idx_pitches` ON `pitches` (
               `pitch`	ASC);";

        let db = sqlite::open(Path::new(db_str)).unwrap();
        db.execute(db_cache_size).unwrap();
        db.execute(db_page_size).unwrap();
        db.execute(db_drop_table).unwrap();
        db.execute("VACUUM;").unwrap();
        db.execute(db_journal_mode).unwrap();
        db.execute(db_synchronous).unwrap();
        db.execute(db_exclusive).unwrap();
        db.execute(db_tempstore).unwrap();
        db.execute(db_create_table).unwrap();
        db.execute(db_create_idx).unwrap();

        let gears: Vec<i32> = gears_from_file(lib);
        let ratios: Vec<f64> = ratios_from_file(gb_ratios);

        // iterate every ab combination
        db.execute("BEGIN").unwrap();
        for combos in gears.clone().into_iter().combinations(2)
        {
            let a = combos[0] as f64;
            let b = combos[1] as f64;

            // generate a list with 2 gears
            // a/b
            // b/a
            for r in &ratios
            {
                // TODO: turn the dbg.write_fmt() calls into functions, or use a string for the
                // format string. FUGLY!
                let mut ans :f64 = mnp * (a/b) * r;
                db.execute(format!("INSERT INTO pitches (pitch,m,n,a,b,c,d,r) VALUES ('{}',{},{},{},{},0,0,{})",ans,m,n,a,b,r)).unwrap();

                ans = mnp * (b/a) * r;
                db.execute(format!("INSERT INTO pitches (pitch,m,n,a,b,c,d,r) VALUES ('{}',{},{},{},{},0,0,{})",ans,m,n,b,a,r)).unwrap();
            }
        }

        // iterate every abcd combination
        for combos in gears.clone().into_iter().combinations(4)
        {
            let a = combos[0] as f64;
            let b = combos[1] as f64;
            let c = combos[3] as f64;
            let d = combos[2] as f64;

            // generate a list with 4 gears
            // 2 on top = 4C2 = 6 sub-combinations:
            // ac / bd
            // ab / cd
            // ad / bc
            // bc / ad
            // bd / ac
            // cd / ab
            for r in &ratios
            {
                // TODO: turn the dbg.write_fmt() calls into functions, or use a string for the
                // format string. FUGLY!
                let mut ans :f64;
                ans = mnp * ((a*c)/(b*d)) * r;
                db.execute(format!("INSERT INTO pitches (pitch,m,n,a,b,c,d,r) VALUES ('{}',{},{},{},{},{},{},{})",ans,m,n,a,c,b,d,r)).unwrap();

                ans = mnp * ((a*b)/(c*d)) * r;
                db.execute(format!("INSERT INTO pitches (pitch,m,n,a,b,c,d,r) VALUES ('{}',{},{},{},{},{},{},{})",ans,m,n,a,b,c,d,r)).unwrap();

                ans = mnp * ((a*d)/(b*c)) * r;
                db.execute(format!("INSERT INTO pitches (pitch,m,n,a,b,c,d,r) VALUES ('{}',{},{},{},{},{},{},{})",ans,m,n,a,d,b,c,r)).unwrap();

                ans = mnp * ((b*c)/(a*d)) * r;
                db.execute(format!("INSERT INTO pitches (pitch,m,n,a,b,c,d,r) VALUES ('{}',{},{},{},{},{},{},{})",ans,m,n,b,c,a,d,r)).unwrap();

                ans = mnp * ((b*d)/(a*c)) * r;
                db.execute(format!("INSERT INTO pitches (pitch,m,n,a,b,c,d,r) VALUES ('{}',{},{},{},{},{},{},{})",ans,m,n,b,d,a,c,r)).unwrap();

                ans = mnp * ((c*d)/(a*b)) * r;
                db.execute(format!("INSERT INTO pitches (pitch,m,n,a,b,c,d,r) VALUES ('{}',{},{},{},{},{},{},{})",ans,m,n,c,d,a,b,r)).unwrap();
            }
        }
        db.execute("END").unwrap();
    }
}

fn lookup(m_opts: clap::ArgMatches)
{
    if let Some(ref m_opts) = m_opts.subcommand_matches("lookup")
    {
        // db output file string
        let db_str = m_opts.value_of("db").unwrap();
        let db = sqlite::open(Path::new(db_str)).unwrap();
        let pitch :f64 = m_opts.value_of("pitch").unwrap().parse().unwrap();  // this is just derp

        // something panics around here
        let mut cursor = db
            .prepare("SELECT * FROM pitches WHERE pitch = ?")
            .unwrap()
            .cursor();

        println!("pitch = {}", pitch);
        cursor.bind(&[Value::Float(pitch)]).unwrap();

        while let Some(row) = cursor.next().unwrap()
        {
            // or maybe here???
            println!("{} = {}/{} * {}/{} * {}/{} * {}",
                row[1].as_float().unwrap(),
                row[2].as_integer().unwrap(),
                row[3].as_integer().unwrap(),
                row[4].as_integer().unwrap(),
                row[5].as_integer().unwrap(),
                row[6].as_integer().unwrap(),
                row[7].as_integer().unwrap(),
                row[8].as_float().unwrap()
            );
        }
    }
}

fn gears_from_file<P>(filename: P) -> Vec<i32>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).expect("No such file.");
    let gears = BufReader::new(file);
    gears.lines()
        .map(|l| l.expect("could not parse line"))
        .map(|l| l.parse::<i32>().unwrap())
        .collect()
}

fn ratios_from_file<P>(filename: P) -> Vec<f64>
where
    P: AsRef<Path>,
{
    use meval::eval_str;
    let file = File::open(filename).expect("No such file.");
    let ratios = BufReader::new(file);
    ratios.lines()
        .map(|l| l.expect("could not parse line"))
        .map(|l| match eval_str(l)
        {
            Ok(v) => v,
            Err(e) => panic!("{:?}", e)
        })
        .collect()
}

#[cfg(not(feature = "yaml"))]
fn main() {
    // As stated above, if clap is not compiled with the YAML feature, it is disabled.
    println!("YAML feature is disabled.");
    println!("Pass --features yaml to cargo when trying this example.");
}
