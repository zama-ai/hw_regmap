use tera::Tera;

use super::regmap::parser::{Owner, ReadAccess, WriteAccess};
use super::regmap::Register;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SvRegister {
    name: String,
    param_snippets: String,
    io_snippets: String,
    rd_snippets: String,
    ff_wr_snippets: String,
}

impl SvRegister {
    pub fn from_register(
        section_name: &str,
        register_name: &str,
        register_props: &Register,
        tera: &Tera,
    ) -> Self {
        let mut context = tera::Context::new();
        let full_name = format!("{section_name}_{register_name}");
        context.insert("name", &full_name);
        context.insert("offset", register_props.offset());
        let (dn, dv) = register_props.default().to_sv_namesval();
        context.insert("default_name", &dn);
        context.insert("default_val", &dv);
        // Expand Owner/Mode to ease tera templating
        context.insert(
            "param_reg",
            &matches!(register_props.owner(), Owner::Parameter),
        );
        context.insert(
            "reg_update",
            &matches!(register_props.owner(), Owner::Kernel | Owner::Both),
        );
        context.insert(
            "wr_user",
            &match register_props.owner() {
                Owner::User | Owner::Both => register_props.write_access() != &WriteAccess::None,
                _ => false,
            },
        );
        context.insert(
            "rd_notify",
            &matches!(register_props.read_access(), ReadAccess::ReadNotify),
        );
        context.insert(
            "wr_notify",
            &matches!(
                register_props.write_access(),
                WriteAccess::WriteNotify | WriteAccess::WriteAction
            ),
        );
        context.insert(
            "wr_action",
            &matches!(register_props.write_access(), WriteAccess::WriteAction),
        );

        // Render Param section
        let param_snippets = match register_props.owner() {
            Owner::Parameter => tera.render("module/param.sv", &context).unwrap(),
            _ => String::new(),
        };

        // Render Io section
        let io_snippets = match register_props.owner() {
            Owner::Parameter => String::new(),
            _ => tera.render("module/io.sv", &context).unwrap(),
        };

        let ff_wr_snippets = tera.render("module/write.sv", &context).unwrap();

        let rd_snippets = match register_props.read_access() {
            ReadAccess::None => String::new(),
            ReadAccess::Read | ReadAccess::ReadNotify => {
                tera.render("module/read.sv", &context).unwrap()
            }
        };
        Self {
            name: full_name,
            param_snippets,
            io_snippets,
            rd_snippets,
            ff_wr_snippets,
        }
    }
}
