use std::collections::HashMap;

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

/// Simple function used to render integer in hexadecimal format with tera
/// Syntax in tera file is: as_hex()
pub fn as_hex(args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
    // Extract width if specified
    let width = if let Some(width) = args.get("width") {
        if let tera::Value::Number(num) = width {
            Ok(num.as_u64().unwrap() as usize)
        } else {
            Err(tera::Error::msg("Width is not an integer"))
        }
    } else {
        Ok(0)
    }?;

    if let Some(value) = args.get("val") {
        if let tera::Value::Number(num) = value {
            let hex_str = format!("0x{:0>width$x}", num.as_u64().unwrap(), width = width);
            Ok(tera::Value::String(hex_str))
        } else {
            Err(tera::Error::msg("Value is not an integer"))
        }
    } else {
        Err(tera::Error::msg(
            "Function `as_hex` didn't receive a `val` argument",
        ))
    }
}

fn main() -> std::io::Result<()> {
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

    // Create a new Tera instances
    // Analyse all available SystemVerilog template
    let tera_sv = Tera::new("templates/**/*.sv").unwrap();
    // Analyse all available doc template
    let mut tera_doc = Tera::new("templates/**/fmt_as.*").unwrap();
    tera_doc.register_function("as_hex", as_hex);

    // Ensure that output folder exist
    std::fs::create_dir_all(&args.output_path).unwrap();

    // Generate module body  ======================================================================
    let rtl_module = format!("{}/{}.sv", args.output_path, args.basename);

    // Convert regmap in rtl snippets based on Tera
    let mut regs_sv = Vec::new();
    let mut used_params = Vec::new();
    regmap.section().iter().for_each(|sec| {
        sec.register().iter().for_each(|reg| {
            regs_sv.push(generator::SvRegister::from_register(
                sec.name(),
                reg,
                &mut used_params,
                &tera_sv,
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
    let module_rendered = tera_sv.render("module.sv", &context).unwrap();
    let module_post_rendered = post_process(&module_rendered);

    std::fs::write(rtl_module, module_post_rendered).expect("Unable to write file");

    // Generate addr/field pkg ===================================================================
    let rtl_pkg = format!("{}/{}_pkg.sv", args.output_path, args.basename);

    // Convert regmap in pkg snippets based on Tera
    let mut regs_pkg_sv = Vec::new();
    regmap.section().iter().for_each(|sec| {
        sec.register().iter().for_each(|reg| {
            regs_pkg_sv.push(generator::SvRegisterPkg::from_register(
                sec.name(),
                regmap.word_size_b(),
                reg,
                &tera_sv,
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
    let pkg_rendered = tera_sv.render("pkg.sv", &context).unwrap();
    let pkg_post_rendered = post_process(&pkg_rendered);

    std::fs::write(rtl_pkg, pkg_post_rendered).expect("Unable to write file");

    // Generate documentation ====================================================================
    // JSON
    // Serialize as json for full access all fields
    let doc_json = format!("{}/{}_doc.json", args.output_path, args.basename);
    let doc_json_f = std::fs::File::create(&doc_json)?;
    serde_json::to_writer_pretty(doc_json_f, &regmap)?;

    // Markdown
    // Generate a structure document targeting online documentation
    let doc_md = format!("{}/{}_doc.md", args.output_path, args.basename);
    // Expand to docs and store in targeted file
    let mut context = tera::Context::new();
    // Extract version from env
    let git_version = option_env!("GIT_VERSION").unwrap_or("unknow");
    context.insert("tool_version", git_version);
    context.insert("regmap", &regmap);
    let md_rendered = tera_doc.render("docs/fmt_as.md", &context).unwrap();

    std::fs::write(doc_md, md_rendered).expect("Unable to write file");
    Ok(())
}
