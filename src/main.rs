mod generator;
mod regmap;

use tera::Tera;

/// Define CLI arguments
use clap::Parser;
#[derive(clap::Parser, Debug, Clone)]
#[clap(long_about = "Generate RTL register map")]
pub struct Args {
    // Regmap configuration ----------------------------------------------------
    #[clap(long, value_parser, default_value = "regmap.toml")]
    toml_file: String,

    // Output configuration ----------------------------------------------------
    // Filename of regmap RTL module
    #[clap(long, value_parser, default_value = "output/regmap.sv")]
    rtl_module: Option<String>,

    // Filename of regmap field package
    #[clap(long, value_parser, default_value = "output/regmap_field_pkg.sv")]
    rtl_field_pkg: Option<String>,

    // Filename of regmap Tb lookup
    #[clap(long, value_parser, default_value = "output/regmap_tb_lookup.sv")]
    tb_lookup: Option<String>,

    // Filename of regmap XDC constraints
    #[clap(long, value_parser, default_value = "output/regmap.sv")]
    xdc_ctrs: Option<String>,

    // Debug options ----------------------------------------------------------
    /// Enable verbosity
    #[clap(long, value_parser)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    println!("User Options: {args:?}");

    // Parse toml file
    let regmap = regmap::parser::RegmapOpt::read_from(&args.toml_file);

    // Expand regmap => Check properties and expand optionnal fields
    let regmap = regmap::Regmap::from_opt(regmap).unwrap();
    if args.verbose {
        println!("{regmap}");
    }

    // Create a new Tera instance
    // Analyse all available template
    let tera = Tera::new("templates/**/*").unwrap();

    if let Some(rtl_module) = args.rtl_module {
        // Convert regmap in rtl snippets based on Tera
        let mut regs_sv = Vec::new();
        regmap.section().iter().for_each(|(sec_name, sec)| {
            sec.register().iter().for_each(|(reg_name, reg)| {
                regs_sv.push(generator::SvRegister::from_register(
                    sec_name, reg_name, reg, &tera,
                ));
            })
        });

        // Expand to rtl module and store in targeted file
        let mut context = tera::Context::new();
        // Extract version from env
        let git_version = option_env!("GIT_VERSION").unwrap_or("unknow");
        context.insert("tool_version", git_version);
        context.insert("name", "build_my_name"); // TODO
        context.insert("word_size_b", &regmap.word_size_b());
        context.insert("offset", &regmap.offset());
        context.insert("range", &regmap.range());
        context.insert("regs_sv", &regs_sv);
        let module_rendered = tera.render("module.sv", &context).unwrap();

        std::fs::write(rtl_module, module_rendered).expect("Unable to write file");
    }
}
