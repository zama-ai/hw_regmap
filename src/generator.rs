use tera::Tera;

use super::regmap::parser::{Owner, ReadAccess, WriteAccess};
use super::regmap::Register;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SvRegister {
    name: String,
    param_snippets: String,
    io_snippets: String,
    default_snippets: String,
    rd_snippets: String,
    ff_wr_snippets: String,
}

impl SvRegister {
    pub fn from_register(
        section_name: &str,
        register: &Register,
        used_params: &mut Vec<String>,
        tera: &Tera,
    ) -> Self {
        let mut context = tera::Context::new();
        let full_name = format!("{section_name}_{}", register.name());
        let mut cst_name = format!("{section_name}_{}_OFS", register.name());
        cst_name.make_ascii_uppercase();
        context.insert("name", &full_name);
        context.insert("offset_cst_name", &cst_name);
        let mut dflt_name = register.default().params_list();
        // Filter duplication in param_name.
        // NB: A parameters used by multiple reg must appear only once at top level
        // -> Retain only params not already in use and update the in-use list
        dflt_name.retain(|e| !used_params.contains(e));
        used_params.extend(dflt_name.clone());

        context.insert("default_name", &dflt_name);
        context.insert("default_val", register.default());
        // Expand Owner/Mode to ease tera templating
        context.insert("param_reg", &matches!(register.owner(), Owner::Parameter));
        context.insert("reg_update", &matches!(register.owner(), Owner::Kernel));
        context.insert(
            "wr_user",
            &match register.owner() {
                Owner::User => register.write_access() != &WriteAccess::None,
                _ => false,
            },
        );
        context.insert(
            "rd_notify",
            &matches!(register.read_access(), ReadAccess::ReadNotify),
        );
        context.insert(
            "wr_notify",
            &matches!(register.write_access(), WriteAccess::WriteNotify),
        );

        context.insert("have_fields", &register.field().is_some());

        // Render Param section
        // NB: Trim \n at end to prevent double comma insertion
        let raw_param_snippets = tera.render("module/param.sv", &context).unwrap();
        let param_snippets = raw_param_snippets.trim_end_matches("\n").to_string();

        // Render Io section
        let io_snippets = match register.owner() {
            Owner::Parameter => String::new(),
            _ => tera.render("module/io.sv", &context).unwrap(),
        };

        let default_snippets = tera.render("module/default.sv", &context).unwrap();

        let ff_wr_snippets = tera.render("module/write.sv", &context).unwrap();

        let rd_snippets = match register.read_access() {
            ReadAccess::None => String::new(),
            ReadAccess::Read | ReadAccess::ReadNotify => {
                tera.render("module/read.sv", &context).unwrap()
            }
        };
        Self {
            name: full_name,
            param_snippets,
            io_snippets,
            default_snippets,
            rd_snippets,
            ff_wr_snippets,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SvRegisterPkg {
    name: String,
    description: String,
    addr_snippets: String,
    struct_snippets: String,
}

impl SvRegisterPkg {
    pub fn from_register(
        section_name: &str,
        word_w: &usize,
        register: &Register,
        tera: &Tera,
    ) -> Self {
        let mut context = tera::Context::new();
        let base_name = format!("{section_name}_{}", register.name());
        let mut ofs_name = format!("{base_name}_OFS");
        ofs_name.make_ascii_uppercase();
        context.insert("base_name", &base_name);
        context.insert("ofs_name", &ofs_name);
        context.insert("ofs_val", &format!("'h{:x}", register.offset()));

        if let Some(fields) = register.field() {
            // Sanitize fields -> insert padding if necessary
            let mut cur_ofs = 0;
            let mut padded_fields = Vec::new();
            for f in fields {
                if cur_ofs != *f.offset_b() {
                    padded_fields.push((
                        format!("padding_{cur_ofs}"),
                        cur_ofs,
                        (f.offset_b() - cur_ofs),
                    ));
                }
                padded_fields.push((f.name().clone(), *f.offset_b(), *f.size_b()));
                cur_ofs = f.offset_b() + f.size_b();
            }
            if cur_ofs != *word_w {
                padded_fields.push((format!("padding_{cur_ofs}"), cur_ofs, (word_w - cur_ofs)));
            }
            // NB: SystemVerilog struct are defined from MSB word to LSB word
            padded_fields.reverse();
            context.insert("fields_nos", &padded_fields);
        }

        // Render addr section
        let addr_snippets = tera.render("pkg/addr.sv", &context).unwrap();

        // Render struct section
        let struct_snippets = if register.field().is_some() {
            tera.render("pkg/struct.sv", &context).unwrap()
        } else {
            String::new()
        };

        Self {
            name: base_name,
            description: register.description().clone(),
            addr_snippets,
            struct_snippets,
        }
    }
}
