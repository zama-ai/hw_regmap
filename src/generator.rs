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
        let mut cst_name = format!("{section_name}_{register_name}_REG_OFS");
        cst_name.make_ascii_uppercase();
        context.insert("name", &full_name);
        context.insert("offset_cst_name", &cst_name);
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
            &matches!(register_props.write_access(), WriteAccess::WriteNotify),
        );

        context.insert("have_fields", &register_props.field().is_some());

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
        register_name: &str,
        register_w: &usize,
        register_props: &Register,
        tera: &Tera,
    ) -> Self {
        let mut context = tera::Context::new();
        let base_name = format!("{section_name}_{register_name}");
        let mut ofs_name = format!("{base_name}_REG_OFS");
        ofs_name.make_ascii_uppercase();
        context.insert("base_name", &base_name);
        context.insert("ofs_name", &ofs_name);
        context.insert("ofs_val", &register_props.offset());

        if let Some(fields) = register_props.field() {
            // Sanitize fields -> insert padding if necessary
            let mut cur_ofs = 0;
            let mut padded_fields = Vec::new();
            for (k, v) in fields {
                if cur_ofs != *v.offset_b() {
                    padded_fields.push((
                        format!("padding_{cur_ofs}"),
                        cur_ofs,
                        (v.offset_b() - cur_ofs),
                    ));
                }
                padded_fields.push((k.clone(), *v.offset_b(), *v.size_b()));
                cur_ofs = v.offset_b() + v.size_b();
            }
            if cur_ofs != *register_w {
                padded_fields.push((
                    format!("padding_{cur_ofs}"),
                    cur_ofs,
                    (register_w - cur_ofs),
                ));
            }
            // NB: SystemVerilog struct are defined from MSB word to LSB word
            padded_fields.reverse();
            context.insert("fields_nos", &padded_fields);
        }

        // Render addr section
        context.insert("ofs_name", &ofs_name);
        context.insert("ofs_val", &register_props.offset());
        let addr_snippets = tera.render("pkg/addr.sv", &context).unwrap();

        // Render struct section
        let struct_snippets = if let Some(fields) = register_props.field() {
            tera.render("pkg/struct.sv", &context).unwrap()
        } else {
            String::new()
        };

        Self {
            name: base_name,
            description: register_props.description().clone(),
            addr_snippets,
            struct_snippets,
        }
    }
}
