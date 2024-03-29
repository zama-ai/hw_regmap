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
        context.insert("default", &register_props.default().to_sv_string());

        // Render Param section
        let param_snippets = match register_props.owner() {
            Owner::Parameter => tera.render("param/param.sv", &context).unwrap(),
            _ => String::new(),
        };

        // Render Io section
        // let io_snippets = match register_props.owner() {
        //     Owner::Parameter => String::new(),
        //     Owner::Parameter => tera.render("param.sv", &context).unwrap(),
        //     _ => String::new(),
        // };
        let io_snippets = String::new();

        let ff_wr_snippets = match register_props.write_access() {
            WriteAccess::None => tera.render("write/none.sv", &context).unwrap(),
            WriteAccess::Write => tera.render("write/write.sv", &context).unwrap(),
            WriteAccess::WriteNotify => tera.render("write/write_notify.sv", &context).unwrap(),
            WriteAccess::WriteAction => tera.render("write/write_action.sv", &context).unwrap(),
        };

        let rd_snippets = match register_props.read_access() {
            ReadAccess::None => String::new(),
            ReadAccess::Read => tera.render("read/read.sv", &context).unwrap(),
            ReadAccess::ReadNotify => tera.render("read/read_notify.sv", &context).unwrap(),
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
