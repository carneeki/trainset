#[macro_use]
extern crate clap;
extern crate meval;
extern crate itertools;

use std::io::{self, BufRead};
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::fs::File;
use itertools::Itertools;

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

        // file output (w/buffering)
        let out_str = m_opts.value_of("out").unwrap();
        let mut out_file = match File::create(Path::new(out_str))
        {
            Ok(out_file) => out_file,
            Err(e) => panic!("Couldn't open output file: {:?}", e),
        };
        let mut out = BufWriter::new(&out_file);

        let gears: Vec<i32> = gears_from_file(lib);
        let ratios: Vec<f64> = ratios_from_file(gb_ratios);

        let mnp: f64 = (m/n)*pitch;

        // iterate every ab combination
        for combos in gears.clone().into_iter().combinations(2)
        {
            let a = combos[0] as f64;
            let b = combos[1] as f64;

            // generate a list with 2 gears
            // a/b
            // b/a
            for r in &ratios
            {
                let mut ans = (mnp * (a/b) * r);
                out.write_fmt(format_args!("({:03}/{:03}) * ({:03} / {:03}) * {:2.5} * {:?} = {:2.5}mm ({:2.5} TPI)\n", m, n, a, b, r, pitch, ans, 25.4/ans ));
                ans = (mnp * (b/a) * r);
                out.write_fmt(format_args!("({:03}/{:03}) * ({:03} / {:03}) * {:2.5} * {:?} = {:2.5}mm ({:2.5} TPI)\n", m, n, a, b, r, pitch, ans, 25.4/ans ));
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
                let mut ans;
                ans = (mnp * ((a*c)/(b*d)) * r);
                out.write_fmt(format_args!("({:03}/{:03}) * [({:03} * {:03}) / ({:03} * {:03})] * {:2.5} * {:?} = {:2.5}mm ({:2.5} TPI)\n", m, n, a, c, b, d, r, pitch, ans, 25.4/ans) );
                ans = (mnp * ((a*b)/(c*d)) * r);
                out.write_fmt(format_args!("({:03}/{:03}) * [({:03} * {:03}) / ({:03} * {:03})] * {:2.5} * {:?} = {:2.5}mm ({:2.5} TPI)\n", m, n, a, b, c, d, r, pitch, ans, 25.4/ans) );
                ans = (mnp * ((a*d)/(b*c)) * r);
                out.write_fmt(format_args!("({:03}/{:03}) * [({:03} * {:03}) / ({:03} * {:03})] * {:2.5} * {:?} = {:2.5}mm ({:2.5} TPI)\n", m, n, a, d, b, c, r, pitch, ans, 25.4/ans) );
                ans = (mnp * ((b*c)/(a*d)) * r);
                out.write_fmt(format_args!("({:03}/{:03}) * [({:03} * {:03}) / ({:03} * {:03})] * {:2.5} * {:?} = {:2.5}mm ({:2.5} TPI)\n", m, n, b, c, a, d, r, pitch, ans, 25.4/ans) );
                ans = (mnp * ((b*d)/(a*c)) * r);
                out.write_fmt(format_args!("({:03}/{:03}) * [({:03} * {:03}) / ({:03} * {:03})] * {:2.5} * {:?} = {:2.5}mm ({:2.5} TPI)\n", m, n, b, d, a, c, r, pitch, ans, 25.4/ans) );
                ans = (mnp * ((c*d)/(a*b)) * r);
                out.write_fmt(format_args!("({:03}/{:03}) * [({:03} * {:03}) / ({:03} * {:03})] * {:2.5} * {:?} = {:2.5}mm ({:2.5} TPI)\n", m, n, c, d, a, b, r, pitch, ans, 25.4/ans) );
                /*
                print_abcd(&m, &n, &a, &c, &b, &d, &r, &pitch, mnp * ( (a*c) / (b*d)) * r );
                print_abcd(&m, &n, &a, &b, &c, &d, &r, &pitch, mnp * ( (a*b) / (c*d)) * r );
                print_abcd(&m, &n, &a, &d, &b, &c, &r, &pitch, mnp * ( (a*d) / (b*c)) * r );
                print_abcd(&m, &n, &b, &c, &a, &d, &r, &pitch, mnp * ( (b*c) / (a*d)) * r );
                print_abcd(&m, &n, &b, &d, &a, &c, &r, &pitch, mnp * ( (b*d) / (a*c)) * r );
                print_abcd(&m, &n, &c, &d, &a, &b, &r, &pitch, mnp * ( (c*d) / (a*b)) * r );
                */
            }
        }
    }
}

//fn print_abcd(m: &f64, n: &f64, a: &f64, b: &f64, c: &f64, d: &f64, r: &f64, pitch: &f64, ans: f64)
//{
    //println!("({:03}/{:03}) * [({:03} * {:03}) / ({:03} * {:03})] * {:2.5} * {:?} = {:2.5}", m, n, a, c, b, d, r, pitch, ans );
//}

fn lookup(m: clap::ArgMatches)
{
    if let Some(ref m) = m.subcommand_matches("generate")
    {
        let db = m.value_of("db").unwrap();
        let pitch = m.value_of("pitch").unwrap();
        for line in lines_from_file(db).unwrap()
        {
            let line = match line
            {
                Ok(line) => line,
                Err(err) => panic!("failed to read line: {}", err)
            };
            assert_eq!(line.trim(), pitch);
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
        //.map(|l| l.parse::<f64>().unwrap())
        .collect()
}

// https://stackoverflow.com/a/30801708
// returns BufReader s of strings
fn lines_from_file<P>(filename: P) -> Result<io::Lines<io::BufReader<File>>, io::Error>
where
    P: AsRef<Path>,
{
    let file = try!(File::open(filename));
    Ok(io::BufReader::new(file).lines())
}

#[cfg(not(feature = "yaml"))]
fn main() {
    // As stated above, if clap is not compiled with the YAML feature, it is disabled.
    println!("YAML feature is disabled.");
    println!("Pass --features yaml to cargo when trying this example.");
}
