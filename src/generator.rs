use std::default;

use tera::Tera;

use super::regmap::parser::{Owner, ReadAccess, WriteAccess};
use super::regmap::Section;
use super::regmap::Register;
use super::regmap::Field;
use super::regmap::DefaultVal;

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


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SvRalSection {
    name: String,
    sctd_snippet: String,
    offset : String
}

impl SvRalSection {
    pub fn new(name: String, sctd_snippet: String, offset: String) -> Self {
        Self { name, sctd_snippet, offset }
    }

    pub fn from_section(
        section: &Section,
        tera: &Tera,
    ) -> Self {
        let mut context = tera::Context::new();
        let sct_name = format!("{}", section.name()).to_lowercase();
        let mut registers = Vec::new();
        context.insert("section_name", &sct_name);
        for r in section.register(){
            registers.push(r);
        }
        context.insert("registers", &registers);
        context.insert("section", &section);
        let sctd_snippet = tera.render("ral/ral_sct_dclr.sv", &context).unwrap();

        Self { name: sct_name
             , sctd_snippet: sctd_snippet
             , offset: format!("{:x}", section.offset())
             }
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SvRalReg {
    name: String,
    regd_snippet: String,
    offset : String
}


fn make_sv_fields(register: &Register, fallback_name: &str) -> Vec<SvRalField> {
    if let Some(fields) = register.field() {
        fields.iter().map(SvRalField::from_field).collect()
    } else {
        vec![SvRalField {
            name: fallback_name.to_string(),
            reset: Some(register.default().clone()),
            access: match (register.read_access(), register.write_access()) {
                (ReadAccess::Read, WriteAccess::Write) => SvRAlAccess::RW,
                (ReadAccess::None, WriteAccess::Write) => SvRAlAccess::WO,
                (ReadAccess::Read, WriteAccess::None)  => SvRAlAccess::RO,
                (ReadAccess::None, WriteAccess::None)  => panic!("Can't have unaccessible registers"),
                (_, WriteAccess::WriteNotify) => SvRAlAccess::RW,
                (ReadAccess::ReadNotify, _) => SvRAlAccess::RW,
            },
            ..Default::default()
        }]
    }
}

impl SvRalReg {
    pub fn from_register(
        section: &Section,
        register: &Register,
        tera: &Tera,
    ) -> Self {
        let mut context = tera::Context::new();
        let reg_name = format!("{}_{}", section.name().to_lowercase(), register.name().to_lowercase());
        context.insert("reg_name", &reg_name);
        let sv_fields = make_sv_fields(register, &reg_name);
        context.insert("sv_fields", &sv_fields);

        let regd_snippet = tera.render("ral/ral_reg_dclr.sv", &context).unwrap();
        Self {
            name: reg_name,
            regd_snippet: regd_snippet,
            offset: format!("{:x}", register.offset()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SvRAlAccess{
  RO,
  RW,
  RC,
  RS,
  WRC,
  WRS,
  WC,
  WS,
  WSRC,
  WCRS,
  W1C,
  W1S,
  W1T,
  W0C,
  W0S,
  W0T,
  W1SRC,
  W1CRS,
  W0SRC,
  W0CRS,
  WO,
  WOC,
  WOS,
  W1,
  WO1,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SvRalField{
    name: String,
    size: String,
    lsb_pos: String,
    access: SvRAlAccess,
    volatile: String,
    reset: Option<DefaultVal>,
    is_rand: String,
    individually_accessible: String,
}

impl SvRAlAccess {
  fn as_str(&self) -> &'static str {
    match self {
      SvRAlAccess::RO    => "RO",
      SvRAlAccess::RW    => "RW",
      SvRAlAccess::RC    => "RC",
      SvRAlAccess::RS    => "RS",
      SvRAlAccess::WRC   => "WRC",
      SvRAlAccess::WRS   => "WRS",
      SvRAlAccess::WC    => "WC",
      SvRAlAccess::WS    => "WS",
      SvRAlAccess::WSRC  => "WSRC",
      SvRAlAccess::WCRS  => "WCRS",
      SvRAlAccess::W1C   => "W1C",
      SvRAlAccess::W1S   => "W1S",
      SvRAlAccess::W1T   => "W1T",
      SvRAlAccess::W0C   => "W0C",
      SvRAlAccess::W0S   => "W0S",
      SvRAlAccess::W0T   => "W0T",
      SvRAlAccess::W1SRC => "W1SRC",
      SvRAlAccess::W1CRS => "W1CRS",
      SvRAlAccess::W0SRC => "W0SRC",
      SvRAlAccess::W0CRS => "W0CRS",
      SvRAlAccess::WO    => "WO",
      SvRAlAccess::WOC   => "WOC",
      SvRAlAccess::WOS   => "WOS",
      SvRAlAccess::W1    => "W1",
      SvRAlAccess::WO1   => "WO1",
    }
  }
}

impl SvRalField {
    pub fn from_field(
        field: &Field,
    ) -> Self {
        Self {
            name: field.name().clone(),
            size: format!("{}", field.size_b()),
            lsb_pos: format!("{}",field.offset_b()),
            reset: field.default().clone(),
            access: match (field.read_access(), field.write_access()) {
              (ReadAccess::Read, WriteAccess::Write) => SvRAlAccess::RW,
              (ReadAccess::None, WriteAccess::Write) => SvRAlAccess::WO,
              (ReadAccess::Read, WriteAccess::None)  => SvRAlAccess::RO,
              (ReadAccess::None, WriteAccess::None)  => panic!("Can't have unaccessible registers"),
              (_, WriteAccess::WriteNotify) => SvRAlAccess::RW,//todo!("WriteNotify not yet implemented"),
              (ReadAccess::ReadNotify, _) => SvRAlAccess::RW,//todo!("ReadNotify not yet implemented"),
            },
            ..Default::default()
        }
    }
}
impl Default for SvRalField {
    fn default() -> Self{
        Self { name: String::new(),
            size: String::from("32"),
            lsb_pos: String::from("0"),
            access: SvRAlAccess::RW,
            volatile: String::from("0"),
            is_rand: String::from("1"),
            reset: None,
            individually_accessible: String::from("0"), }
        }
}

/// Represents all generated classes for one group of duplicate sections.
///
/// For a section `Foo` with `duplicate=["_a","_b"]`, the group has:
///   base_name = "foo"
///   instances = [foo_a_section (thin wrapper), foo_b_section (thin wrapper)]
///   base register class `foo_<reg>_base_reg` holds all field definitions
///   per-instance register classes `foo_a_<reg>_reg` / `foo_b_<reg>_reg` extend it
///   base section class `foo_base_section` references base register types
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SvRalDupGroup {
    pub base_name: String,
    pub base_reg_names: Vec<String>,
    pub base_reg_snippets: Vec<String>,
    pub inst_reg_names: Vec<String>,
    pub inst_reg_snippets: Vec<String>,
    pub base_sct_snippet: String,
    pub instances: Vec<SvRalSection>,
    /// Short register field names (e.g. "cps", "ctrl") used to generate
    /// per-register arrays: `key_cps_list[N]` of type `key_cps_base_reg`.
    pub reg_field_names: Vec<String>,
}

impl SvRalDupGroup {
    pub fn from_sections(
        base_name: &str,
        sections: &[&Section],
        tera: &Tera,
    ) -> Self {
        assert!(!sections.is_empty(), "duplicate group must have at least one section");
        let first = sections[0];

        let mut base_reg_names = Vec::new();
        let mut base_reg_snippets = Vec::new();
        let mut inst_reg_names = Vec::new();
        let mut inst_reg_snippets = Vec::new();
        let mut reg_field_names = Vec::new();

        for reg in first.register() {
            reg_field_names.push(reg.name().to_lowercase());
            // Base register class: reuse ral_reg_dclr.sv with "<base>_<reg>_base" as name
            // -> template emits: class <reg_name>_reg extends uvm_reg
            // -> result:         class foo_myreg_base_reg extends uvm_reg
            let base_reg_name = format!("{}_{}_base", base_name, reg.name().to_lowercase());
            base_reg_names.push(base_reg_name.clone());

            let sv_fields = make_sv_fields(reg, &base_reg_name);
            let mut ctx = tera::Context::new();
            ctx.insert("reg_name", &base_reg_name);
            ctx.insert("sv_fields", &sv_fields);
            base_reg_snippets.push(tera.render("ral/ral_reg_dclr.sv", &ctx).unwrap());

            // Per-instance thin wrapper: class foo_a_myreg_reg extends foo_myreg_base_reg
            for sec in sections {
                let inst_reg_name = format!("{}_{}", sec.name().to_lowercase(), reg.name().to_lowercase());
                inst_reg_names.push(inst_reg_name.clone());

                let mut ctx = tera::Context::new();
                ctx.insert("reg_name", &inst_reg_name);
                ctx.insert("base_reg_name", &base_reg_name);
                inst_reg_snippets.push(tera.render("ral/ral_inst_reg_dclr.sv", &ctx).unwrap());
            }
        }

        // Base section class referencing base register types
        let mut registers: Vec<&Register> = first.register().iter().collect();
        let mut ctx = tera::Context::new();
        ctx.insert("base_name", base_name);
        ctx.insert("registers", &registers);
        let base_sct_snippet = tera.render("ral/ral_base_sct_dclr.sv", &ctx).unwrap();

        // Per-instance thin section wrappers
        let mut instances = Vec::new();
        for sec in sections {
            let sec_name = sec.name().to_lowercase();
            let mut ctx = tera::Context::new();
            ctx.insert("section_name", &sec_name);
            ctx.insert("base_name", base_name);
            let inst_sct_snippet = tera.render("ral/ral_inst_sct_dclr.sv", &ctx).unwrap();
            instances.push(SvRalSection::new(
                sec_name,
                inst_sct_snippet,
                format!("{:x}", sec.offset()),
            ));
        }

        Self {
            base_name: base_name.to_string(),
            base_reg_names,
            base_reg_snippets,
            inst_reg_names,
            inst_reg_snippets,
            base_sct_snippet,
            instances,
            reg_field_names,
        }
    }
}
