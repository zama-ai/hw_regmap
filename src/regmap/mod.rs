pub mod parser;

use indexmap::{map::Iter, IndexMap};

use getset::Getters;
use parser::{Owner, ReadAccess, WriteAccess};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Default parsing error
/// Descibe potential default val/param imcompatible options
#[derive(Error, Debug, Clone)]
pub enum DefaultError {
    #[error(
        "Incompatible default value/params. Only one could be specified at a time:\n  => {self:?}."
    )]
    BothSpecified(usize, String),

    #[error("Expect a Parameter name:\n  => {self:?}.")]
    ExpectParam(String),
    #[error("Expect a default value:\n  => {self:?}.")]
    ExpectValue(String),
    #[error("Default parameter missing.\n")]
    Missing,
    #[error("Redefined at outer level: \n => {self:?}")]
    Override(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Default {
    Val(usize),
    Param(String),
    ParamField { name: Vec<String>, value: String },
}

impl Default {
    pub fn to_sv_namesval(&self) -> (Vec<String>, String) {
        match self {
            Self::Val(val) => (vec![], format!("'h{:x}", val)),
            Self::Param(str) => (vec![str.clone()], str.clone()),
            Self::ParamField { name, value } => (name.clone(), value.clone()),
        }
    }
}

/// Field parsing error
/// Descibe potential field error and imcompatible options
#[derive(Error, Debug, Clone)]
pub enum FieldError {
    #[error("Field definition crossed word-boundary:\n  => {self:?}")]
    WordBoundary(parser::FieldOpt),
}

#[derive(Debug, Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Field {
    description: String,
    size_b: usize,
    offset_b: usize,
    default: Default,
}

impl Field {
    pub fn from_opt(
        fields: &mut Iter<'_, String, parser::FieldOpt>,
        word_size: usize,
    ) -> Result<IndexMap<String, Self>, anyhow::Error> {
        let mut expanded_field = IndexMap::new();
        let mut nxt_offset = 0;
        for (name, field) in fields {
            let offset_b = match field.offset_b {
                Some(ofst) => ofst,
                None => nxt_offset,
            };

            if (offset_b + field.size_b) > word_size {
                return Err(FieldError::WordBoundary(field.clone()).into());
            }

            let default = match (field.default_val, &field.param_name) {
                (None, None) => Default::Val(0),
                (None, Some(p)) => Default::Param(p.clone()),
                (Some(v), None) => Default::Val(v),
                (Some(v), Some(p)) => {
                    return Err(DefaultError::BothSpecified(v, p.clone()).into());
                }
            };

            nxt_offset += offset_b + field.size_b;
            let _ = expanded_field.insert(
                name.clone(),
                Self {
                    description: field.description.clone(),
                    size_b: field.size_b,
                    offset_b,
                    default,
                },
            );
        }
        Ok(expanded_field)
    }

    pub fn get_default(
        fields: &IndexMap<String, Self>,
        owner: &Owner,
    ) -> Result<Default, anyhow::Error> {
        match owner {
            Owner::Parameter => {
                let mut params_name = Vec::new();
                let mut params_value = String::new();

                for (k, f) in fields.iter() {
                    let param = match &f.default {
                        Default::Param(p) => p.clone(),
                        _ => {
                            return Err(DefaultError::ExpectParam(k.clone()).into());
                        }
                    };
                    params_value += &format!("+({param} << {})", f.offset_b);
                    params_name.push(param);
                }
                Ok(Default::ParamField {
                    name: params_name,
                    value: params_value,
                })
            }
            _ => {
                let mut value = 0_usize;

                for (k, f) in fields.iter() {
                    match f.default {
                        Default::Val(v) => {
                            value += v << f.offset_b;
                        }
                        _ => {
                            return Err(DefaultError::ExpectValue(k.clone()).into());
                        }
                    }
                }
                Ok(Default::Val(value))
            }
        }
    }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "      description: {}", self.description)?;
        writeln!(
            f,
            "      size_b: {}, offset_b: {}, default: {:?}",
            self.size_b, self.offset_b, self.default
        )?;
        Ok(())
    }
}

/// Register parsing error
/// Descibe potential register error and imcompatible options
#[derive(Error, Debug, Clone)]
pub enum RegisterError {
    #[error("Incompatible Access right for Parameter:\n  => {self:?}")]
    ParameterAccess(parser::RegisterOpt),
    #[error("Incompatible Access right for User:\n  => {self:?}")]
    UserAccess(parser::RegisterOpt),
    #[error("Incompatible Access right for Kernel:\n  => {self:?}")]
    KernelAccess(parser::RegisterOpt),
    #[error("Incompatible Access right for Both:\n  => {self:?}")]
    BothAccess(parser::RegisterOpt),
    #[error("Invalid offset:\n  => {self:?}")]
    Offset(parser::RegisterOpt),
}
#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct Register {
    description: String,
    owner: Owner,
    read_access: ReadAccess,
    write_access: WriteAccess,
    offset: usize,
    default: Default,
    field: Option<IndexMap<String, Field>>,
}

impl Register {
    pub fn from_opt(
        regs: &mut Iter<'_, String, parser::RegisterOpt>,
        offset: usize,
        word_size: usize,
    ) -> Result<IndexMap<String, Self>, anyhow::Error> {
        let mut expanded_register = IndexMap::new();
        let word_bytes = word_size / 8;
        let mut nxt_offset = offset;

        for (name, register) in regs {
            // Check correctness of the mode
            match (register.owner, register.read_access, register.write_access) {
                (Owner::Parameter, ReadAccess::Read, WriteAccess::None) => {}
                (Owner::Parameter, _rd, _wr) => {
                    return Err(RegisterError::ParameterAccess(register.clone()).into())
                }
                (Owner::User, _rd, _wr) => {}
                (Owner::Kernel, _rd, WriteAccess::Write) => {
                    return Err(RegisterError::KernelAccess(register.clone()).into())
                }
                (Owner::Kernel, _rd, _wr) => {}
                (Owner::Both, _rd, WriteAccess::None) => {
                    return Err(RegisterError::BothAccess(register.clone()).into())
                }
                (Owner::Both, _rd, _wr) => {}
            }

            // Check correctness of offset
            let reg_offset = match register.offset {
                Some(ofst) => ofst,
                None => nxt_offset,
            };
            if reg_offset < nxt_offset {
                return Err(RegisterError::Offset(register.clone()).into());
            }

            // Expand inner
            let expand_field = match register.field.as_ref() {
                Some(fields) => {
                    let mut concrete_fields = Field::from_opt(&mut fields.iter(), word_size)?;
                    concrete_fields.sort_by(|_xk, x, _yx, y| x.offset_b.cmp(&y.offset_b));
                    Some(concrete_fields)
                }
                None => None,
            };

            // Expand default
            let default = match register.owner {
                Owner::Parameter => match (register.default_val, &register.param_name) {
                    (None, None) => match expand_field.as_ref() {
                        Some(f) => Field::get_default(f, &register.owner)?,
                        None => {
                            return Err(DefaultError::Missing.into());
                        }
                    },
                    (None, Some(p)) => match expand_field.as_ref() {
                        Some(_f) => {
                            return Err(DefaultError::Override(name.clone()).into());
                        }
                        None => Default::Param(p.clone()),
                    },
                    (Some(_), None) => {
                        return Err(DefaultError::ExpectParam(name.clone()).into());
                    }
                    (Some(v), Some(p)) => {
                        return Err(DefaultError::BothSpecified(v, p.clone()).into());
                    }
                },
                _ => match (register.default_val, &register.param_name) {
                    (None, None) => match expand_field.as_ref() {
                        Some(f) => Field::get_default(f, &register.owner)?,
                        None => Default::Val(0),
                    },
                    (None, Some(_p)) => {
                        return Err(DefaultError::ExpectValue(name.clone()).into());
                    }
                    (Some(v), None) => match expand_field.as_ref() {
                        Some(_f) => {
                            return Err(DefaultError::Override(name.clone()).into());
                        }
                        None => Default::Val(v),
                    },
                    (Some(v), Some(p)) => {
                        return Err(DefaultError::BothSpecified(v, p.clone()).into());
                    }
                },
            };

            // Build register instance
            let mut reg = Self {
                description: register.description.clone(),
                owner: register.owner,
                read_access: register.read_access,
                write_access: register.write_access,
                offset: nxt_offset,
                default,
                field: expand_field,
            };

            // Duplicate if required
            match register.duplicate.as_ref() {
                Some(suffix) => {
                    for s in suffix {
                        let full_name = format!("{}{}", name, s);
                        // patch offset and insert
                        reg.offset = nxt_offset;
                        nxt_offset += word_bytes;
                        let _ = expanded_register.insert(full_name, reg.clone());
                    }
                }
                None => {
                    nxt_offset += word_bytes;
                    let _ = expanded_register.insert(name.clone(), reg.clone());
                }
            }
        }
        Ok(expanded_register)
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "    description: {}", self.description)?;
        writeln!(
            f,
            "    Access: {:?}, {:?}, {:?}, offset: 0x{:x}, default: {:?}",
            self.owner, self.read_access, self.write_access, self.offset, self.default
        )?;
        if self.field.is_some() {
            write!(f, "    Field: [")?;
            for (name, field) in self.field.as_ref().unwrap().iter() {
                write!(f, "\n    {}: [\n", name)?;
                write!(f, "{field}")?;
                write!(f, "    ]")?;
            }
            writeln!(f, "    ]")?;
        }
        Ok(())
    }
}

/// Section parsing error
/// Descibe potential register error and imcompatible options
#[derive(Error, Debug, Clone)]
pub enum SectionError {
    #[error("Invalid offset:\n  => {self:?}")]
    Offset(parser::SectionOpt),
}
#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct Section {
    description: String,
    offset: usize,
    align_offset: bool, // Usefull ?
    register: IndexMap<String, Register>,
}

impl Section {
    pub fn from_opt(
        sections: &mut Iter<'_, String, parser::SectionOpt>,
        offset: usize,
        word_size: usize,
    ) -> Result<IndexMap<String, Self>, anyhow::Error> {
        let mut expanded_section = IndexMap::new();
        let word_bytes = word_size / 8;
        let mut nxt_offset = offset;

        for (name, section) in sections {
            // Check correctness of offset
            let sec_offset = match section.offset {
                Some(ofst) => ofst,
                None => nxt_offset,
            };
            if sec_offset < nxt_offset {
                return Err(SectionError::Offset(section.clone()).into());
            }

            // Duplicate if required
            // Have to regenerate register with updated offset in each duplicated section
            match section.duplicate.as_ref() {
                Some(suffix) => {
                    for s in suffix.iter() {
                        // Expand inner
                        let expanded_reg = Register::from_opt(
                            &mut section.register.iter(),
                            nxt_offset,
                            word_size,
                        )?;

                        let full_name = format!("{}{}", name, s);
                        // update offset and insert
                        nxt_offset += expanded_reg.len() * word_bytes;
                        let _ = expanded_section.insert(
                            full_name,
                            Self {
                                description: section.description.clone(),
                                offset: nxt_offset,
                                align_offset: section.align_offset.unwrap_or(false),
                                register: expanded_reg.clone(),
                            },
                        );
                    }
                }
                None => {
                    // Expand inner
                    let expanded_reg =
                        Register::from_opt(&mut section.register.iter(), nxt_offset, word_size)?;

                    // update offset and insert
                    nxt_offset += expanded_reg.len() * word_bytes;
                    let _ = expanded_section.insert(
                        name.clone(),
                        Self {
                            description: section.description.clone(),
                            offset: nxt_offset,
                            align_offset: section.align_offset.unwrap_or(false),
                            register: expanded_reg.clone(),
                        },
                    );
                }
            }
        }
        Ok(expanded_section)
    }
}

impl std::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  description: {}", self.description)?;
        writeln!(
            f,
            "  offset: 0x{:x}, align_offset: {:?}",
            self.offset, self.align_offset
        )?;
        write!(f, "  Register: [")?;
        for (name, reg) in self.register.iter() {
            write!(f, "\n  {}: [\n", name)?;
            write!(f, "{reg}")?;
            write!(f, "  ]")?;
        }
        writeln!(f, "  ]")?;
        Ok(())
    }
}

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct Regmap {
    module_name: String,
    description: String,
    word_size_b: usize,
    offset: usize,
    ext_pkg: Vec<String>,
    range: usize,
    section: IndexMap<String, Section>,
}

impl Regmap {
    pub fn from_opt(regmap: parser::RegmapOpt) -> Result<Self, anyhow::Error> {
        let offset = regmap.offset.unwrap_or(0);
        let section = Section::from_opt(&mut regmap.section.iter(), offset, regmap.word_size_b)?;
        let range = (regmap.word_size_b / 8)
            * section
                .iter()
                .map(|(_sn, s)| s.register.iter().map(|(_rn, r)| r.offset).max().unwrap())
                .max()
                .unwrap();
        Ok(Self {
            module_name: regmap.module_name,
            description: regmap.description,
            word_size_b: regmap.word_size_b,
            ext_pkg: regmap.ext_pkg,
            offset,
            range,
            section,
        })
    }
}

impl std::fmt::Display for Regmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "description: {}", self.description)?;
        writeln!(
            f,
            "offset: 0x{:x}, word_size_b: {:?}",
            self.offset, self.word_size_b
        )?;
        writeln!(f, "External package: {:?}", self.ext_pkg)?;
        write!(f, "Section: [")?;
        for (name, sec) in self.section.iter() {
            write!(f, "\n{}: [\n", name)?;
            write!(f, "{sec}")?;
            write!(f, "]")?;
        }
        writeln!(f, "]")?;
        Ok(())
    }
}
