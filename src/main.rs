use hw_regmap::generator;
use hw_regmap::regmap;

use regex::Regex;
use tera::Tera;

/// Define CLI arguments
use clap::Parser;
#[derive(clap::Parser, Debug, Clone)]
#[clap(long_about = "Generate RTL register map")]
pub struct Args {
    // Regmap configuration ----------------------------------------------------
    #[clap(long, value_parser)]
    toml_file: Vec<String>,

    // Output configuration ----------------------------------------------------
    // Output folder path
    #[clap(long, value_parser, default_value = "output")]
    output_path: String,

    // Basename of the generated file
    #[clap(long, value_parser, default_value = "regmap")]
    basename: String,

    // Debug options ----------------------------------------------------------
    /// Enable verbosity
    #[clap(long, value_parser)]
    verbose: bool,
}

/// Simple post-process of String generated by templating engine
/// Templating introduce a lot of consecutive newlines due to filtering
/// -> This function remove consecutive newline to have something more readable in the end
fn post_process(raw: &str) -> String {
    // Remove extra new-line generated by tera templating
    let regex = Regex::new(r"\s*\n\s*\n+").unwrap();
    let post_rendered = regex.replace_all(raw, "\n");

    post_rendered.to_string()
}

fn main() {
    let args = Args::parse();
    println!("User Options: {args:?}");

    // Parse toml file
    let mut regmap_list = args
        .toml_file
        .iter()
        .map(|toml| regmap::parser::RegmapOpt::read_from(toml))
        .collect::<Vec<_>>();

    // Expand regmap => Check properties and expand optionnal fields
    let regmap = regmap::Regmap::from_opt(&mut regmap_list).unwrap();
    if args.verbose {
        println!("{regmap}");
    }

    // Create a new Tera instance
    // Analyse all available template
    let tera = Tera::new("templates/**/*.sv").unwrap();

    // Ensure that output folder exist
    std::fs::create_dir_all(&args.output_path).unwrap();

    // Generate module body  ======================================================================
    let rtl_module = format!("{}/{}.sv", args.output_path, args.basename);

    // Convert regmap in rtl snippets based on Tera
    let mut regs_sv = Vec::new();
    let mut used_params = Vec::new();
    regmap.section().iter().for_each(|(sec_name, sec)| {
        sec.register().iter().for_each(|(reg_name, reg)| {
            regs_sv.push(generator::SvRegister::from_register(
                sec_name,
                reg_name,
                reg,
                &mut used_params,
                &tera,
            ));
        })
    });

    // Expand to rtl module and store in targeted file
    let mut context = tera::Context::new();
    // Extract version from env
    let git_version = option_env!("GIT_VERSION").unwrap_or("unknow");
    context.insert("tool_version", git_version);
    context.insert("module_name", &regmap.module_name());
    context.insert("word_size_b", &regmap.word_size_b());
    context.insert("offset", &regmap.offset());
    context.insert("ext_pkg", &regmap.ext_pkg());
    context.insert("range", &regmap.range());
    context.insert("regs_sv", &regs_sv);
    let module_rendered = tera.render("module.sv", &context).unwrap();
    let module_post_rendered = post_process(&module_rendered);

    std::fs::write(rtl_module, module_post_rendered).expect("Unable to write file");

    // Generate addr/field pkg ===================================================================
    let rtl_pkg = format!("{}/{}_pkg.sv", args.output_path, args.basename);

    // Convert regmap in pkg snippets based on Tera
    let mut regs_pkg_sv = Vec::new();
    regmap.section().iter().for_each(|(sec_name, sec)| {
        sec.register().iter().for_each(|(reg_name, reg)| {
            regs_pkg_sv.push(generator::SvRegisterPkg::from_register(
                sec_name,
                reg_name,
                regmap.word_size_b(),
                reg,
                &tera,
            ));
        })
    });

    // Expand to rtl module and store in targeted file
    let mut context = tera::Context::new();
    // Extract version from env
    let git_version = option_env!("GIT_VERSION").unwrap_or("unknow");
    context.insert("tool_version", git_version);
    context.insert("module_name", &regmap.module_name());
    context.insert("word_size_b", &regmap.word_size_b());
    context.insert("regs_pkg_sv", &regs_pkg_sv);
    let pkg_rendered = tera.render("pkg.sv", &context).unwrap();
    let pkg_post_rendered = post_process(&pkg_rendered);

    std::fs::write(rtl_pkg, pkg_post_rendered).expect("Unable to write file");
}
