mod regmap;

use std::fs;
use std::process::exit;

fn main() {
    let filename = "hpu_regif.toml";
    let regmap = regmap::parser::RegmapOpt::read_from(&filename);
    // println!("{regmap:?}");

    // Expand regmap
    let regmap = regmap::Regmap::from_opt(regmap).unwrap();
    println!("{regmap}");
}
