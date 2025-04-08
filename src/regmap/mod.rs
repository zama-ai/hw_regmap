pub mod parser;

use std::collections::HashSet;

use indexmap::map::Iter;

use getset::{Getters, MutGetters};
use parser::{Owner, ReadAccess, WriteAccess};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Global parsing error
/// Describe potential register error and incompatible options
#[derive(Error, Debug, Clone)]
pub enum RegmapError {
    #[error("Couldn't generate Regmap from empty RegmapOpt list")]
    NoEntry,
    #[error("Error: Couldn't merge register map with != word_size_b")]
    WordSize,
    #[error("Field definition crossed word-boundary:[ Word width (bits): {word_b}, Field [offset {field_offset}, width {field_b}]]\n  => {msg_info}")]
    WordBoundary {
        word_b: usize,
        field_offset: usize,
        field_b: usize,
        msg_info: String,
    },
    #[error("Default defined at both level (i.e. register & field): \n => {msg_info}")]
    DfltOverride { msg_info: String },
    #[error("Expect Param or Cst [get: {dflt:?}]:\n  => {msg_info:?}.")]
    DfltInvalid { dflt: DefaultVal, msg_info: String },
    #[error("Incompatible Access right for {owner:?} [rd: {rd:?}, wr: {wr:?}]:\n  => {msg_info}")]
    Access {
        owner: Owner,
        rd: ReadAccess,
        wr: WriteAccess,
        msg_info: String,
    },
    #[error(
        "Invalid offset: [Minimal offset: 0x{min_offset:x}, Requested offset: 0x{request_offset:x}]\n  => {msg_info:?}"
    )]
    Offset {
        min_offset: usize,
        request_offset: usize,
        msg_info: String,
    },
    #[error("Invalid range: Range must be a power of two.\n  => {msg_info}")]
    RangePow2 { range: usize, msg_info: String },
    #[error(
        "Invalid range: [Real range: 0x{real_range:x}, Requested range: 0x{request_range:x}]\n  => {msg_info}"
    )]
    Range {
        request_range: usize,
        real_range: usize,
        msg_info: String,
    },
    #[error(
        "Invalid Range: [Range: 0x{range:x}, Offset: 0x{offset:x}] must not overlaps to enable clever addr filtering.\n  => {msg_info}"
    )]
    OffsetRange {
        offset: usize,
        range: usize,
        msg_info: String,
    },
    #[error("Invalid alignment:[Word alignment: 0x{word_align}, Requested alignment: 0x{request_align}]\n  => {msg_info}")]
    ByteAlign {
        word_align: usize,
        request_align: usize,
        msg_info: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum DefaultVal {
    /// Hardcoded value
    Cst(usize),
    /// Value extract from a Rtl parameters
    Param(String),
    /// Value construct from a list of parameters
    ///  * name field: contains the list of used parameters
    ///  * name_val: contains pair of field_name, value
    ParamsField {
        params: Vec<String>,
        name_val: Vec<(String, String)>,
    },
}

impl DefaultVal {
    pub fn params_list(&self) -> Vec<String> {
        match self {
            Self::Cst(_) => vec![],
            Self::Param(str) => vec![str.clone()],
            Self::ParamsField { params, .. } => params.clone(),
        }
    }
}

/// Utility function to compute aligned offset
fn align_on(bytes_align: usize, val: usize) -> usize {
    let remainder = val % bytes_align;

    if 0 != remainder {
        val + bytes_align - remainder
    } else {
        val
    }
}

#[derive(Debug, Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Field {
    name: String,
    description: String,
    size_b: usize,
    offset_b: usize,
    default: Option<DefaultVal>,
}

impl Field {
    pub fn from_opt(
        fields: &mut Iter<'_, String, parser::FieldOpt>,
        word_size: usize,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let mut expanded_field = Vec::with_capacity(fields.len());
        let mut nxt_offset = 0;
        for (name, field) in fields {
            let offset_b = match field.offset_b {
                Some(ofst) => ofst,
                None => nxt_offset,
            };

            if (offset_b + field.size_b) > (word_size * u8::BITS as usize) {
                return Err(RegmapError::WordBoundary {
                    word_b: word_size,
                    field_offset: offset_b,
                    field_b: field.size_b,
                    msg_info: format!("{:?}", field),
                }
                .into());
            }

            nxt_offset += offset_b + field.size_b;
            expanded_field.push(Self {
                name: name.clone(),
                description: field.description.clone(),
                size_b: field.size_b,
                offset_b,
                default: field.default.clone(),
            });
        }
        // Sort by offset_b
        expanded_field.sort_by(|a, b| a.offset_b.cmp(&b.offset_b));

        Ok(expanded_field)
    }

    pub fn get_default(
        fields: &[Self],
        reg_ctx: &parser::RegisterOpt,
    ) -> Result<Option<DefaultVal>, anyhow::Error> {
        let field_with_dflt = fields
            .iter()
            .filter(|field| field.default.is_some())
            .collect::<Vec<_>>();

        if field_with_dflt.is_empty() {
            Ok(None)
        } else {
            let mut params = Vec::new();
            let mut name_val = Vec::new();

            for field in fields.iter() {
                match field.default.as_ref() {
                    Some(DefaultVal::Param(p)) => {
                        // Expose parameters at interface and update name_val
                        params.push(p.clone());
                        name_val.push((field.name.clone(), p.clone()));
                    }
                    Some(DefaultVal::Cst(val)) => {
                        // Update name_val only
                        name_val.push((field.name.clone(), format!("'h{val:x}")));
                    }
                    Some(DefaultVal::ParamsField { .. }) => {
                        return Err(RegmapError::DfltInvalid {
                            dflt: field.default.as_ref().unwrap().clone(),
                            msg_info: format!("{:?}", reg_ctx),
                        }
                        .into());
                    }
                    None => {
                        // Update name_val only with 0 value
                        name_val.push((field.name.clone(), "'h0".to_string()));
                    }
                };
            }
            Ok(Some(DefaultVal::ParamsField { params, name_val }))
        }
    }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "      name: {}", self.name)?;
        writeln!(f, "      description: {}", self.description)?;
        writeln!(
            f,
            "      size_b: {}, offset_b: {}, default: {:?}",
            self.size_b, self.offset_b, self.default
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Register {
    name: String,
    description: String,
    owner: Owner,
    read_access: ReadAccess,
    write_access: WriteAccess,
    offset: usize,
    default: DefaultVal,
    field: Option<Vec<Field>>,
}

impl Register {
    pub fn from_opt(
        regs: &mut Iter<'_, String, parser::RegisterOpt>,
        section_offset: usize,
        word_size: usize,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let mut expanded_register = Vec::with_capacity(regs.len());
        let word_bytes = word_size / std::mem::size_of::<u8>();
        let mut auto_offset = section_offset;

        for (name, register) in regs {
            // Check correctness of the mode
            match (register.owner, register.read_access, register.write_access) {
                (Owner::Parameter, ReadAccess::Read, WriteAccess::None) => {}
                (Owner::Parameter, _rd, _wr) => {
                    return Err(RegmapError::Access {
                        owner: register.owner,
                        rd: register.read_access,
                        wr: register.write_access,
                        msg_info: format!("{:?}", register),
                    }
                    .into())
                }
                (Owner::User, _rd, _wr) => {}
                (Owner::Kernel, _rd, WriteAccess::Write) => {
                    return Err(RegmapError::Access {
                        owner: register.owner,
                        rd: register.read_access,
                        wr: register.write_access,
                        msg_info: format!("{:?}", register),
                    }
                    .into())
                }
                (Owner::Kernel, _rd, _wr) => {}
            }

            // Extract required alignment
            // Subword alignment is not supported
            let bytes_align = match register.bytes_align {
                Some(align) => {
                    if (align % word_bytes) != 0 {
                        return Err(RegmapError::ByteAlign {
                            word_align: word_bytes,
                            request_align: align,
                            msg_info: format!("{:?}", register),
                        }
                        .into());
                    } else {
                        align
                    }
                }
                None => word_bytes,
            };

            // Compute offset with alignment
            let raw_offset = match register.offset {
                Some(ofst) => ofst + section_offset,
                None => auto_offset,
            };
            let mut reg_offset = align_on(bytes_align, raw_offset);

            // Check correctness of offset
            if reg_offset < auto_offset {
                return Err(RegmapError::Offset {
                    min_offset: auto_offset,
                    request_offset: reg_offset,
                    msg_info: format!("{:?}", register),
                }
                .into());
            }

            // Expand inner
            let expand_field = match register.field.as_ref() {
                Some(fields) => {
                    let concrete_fields = Field::from_opt(&mut fields.iter(), word_size)?;
                    Some(concrete_fields)
                }
                None => None,
            };

            // Expand default
            let default = match register.default.as_ref() {
                Some(dflt) => match expand_field.as_ref() {
                    Some(field) => match Field::get_default(field, register)? {
                        Some(_dflt) => {
                            return Err(RegmapError::DfltOverride {
                                msg_info: format!("{:?}", register),
                            }
                            .into());
                        }
                        None => match dflt {
                            DefaultVal::ParamsField { .. } => {
                                return Err(RegmapError::DfltInvalid {
                                    dflt: dflt.clone(),
                                    msg_info: format!("{:?}", register),
                                }
                                .into());
                            }
                            _ => dflt.clone(),
                        },
                    },
                    None => dflt.clone(),
                },
                None => match expand_field.as_ref() {
                    Some(field) => match Field::get_default(field, register)? {
                        Some(dflt) => dflt,
                        None => DefaultVal::Cst(0),
                    },
                    None => DefaultVal::Cst(0),
                },
            };

            // Build register instance
            let mut reg = Self {
                name: name.clone(),
                description: register.description.clone(),
                owner: register.owner,
                read_access: register.read_access,
                write_access: register.write_access,
                offset: reg_offset,
                default,
                field: expand_field,
            };

            // Handle duplication
            // -> No duplication is 1 iteration without name extension
            // NB: Duplication always have automatically computed offset
            let duplicate = register.duplicate.clone().unwrap_or(vec![String::new()]);
            duplicate.iter().enumerate().for_each(|(i, s)| {
                let full_name = format!("{}{}", name, s);
                // Patch name
                reg.name = full_name;

                // Patch offset if needed
                if i != 0 {
                    reg_offset = align_on(bytes_align, reg_offset + word_bytes);
                    reg.offset = reg_offset;
                }
                // Insert in regmap
                expanded_register.push(reg.clone());
            });
            // Update next usable offset
            auto_offset = reg_offset + word_bytes;
        }
        // Sort by offset
        expanded_register.sort_by(|a, b| a.offset.cmp(&b.offset));

        Ok(expanded_register)
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "    name: {}", self.name)?;
        writeln!(f, "    description: {}", self.description)?;
        writeln!(
            f,
            "    Access: {:?}, {:?}, {:?}, offset: 0x{:x}, default: {:?}",
            self.owner, self.read_access, self.write_access, self.offset, self.default
        )?;
        if self.field.is_some() {
            write!(f, "    Field: [")?;
            for field in self.field.as_ref().unwrap().iter() {
                write!(f, "\n[{field}]")?;
            }
            writeln!(f, "    ]")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Section {
    name: String,
    description: String,
    offset: usize,
    bytes_align: usize,
    range: usize,
    register: Vec<Register>,
}

impl Section {
    pub fn from_opt(
        sections: &mut Iter<'_, String, parser::SectionOpt>,
        regmap_offset: usize,
        word_bytes: usize,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let mut expanded_section = Vec::with_capacity(sections.len());
        let mut auto_offset = regmap_offset;

        for (name, section) in sections {
            // Extract required alignment
            // Subword alignment is not supported
            let bytes_align = match section.bytes_align {
                Some(align) => {
                    if (align % word_bytes) != 0 {
                        return Err(RegmapError::ByteAlign {
                            word_align: word_bytes,
                            request_align: align,
                            msg_info: format!("{:?}", section),
                        }
                        .into());
                    } else {
                        align
                    }
                }
                None => word_bytes,
            };

            // Compute offset with alignment
            let raw_offset = match section.offset {
                Some(ofst) => ofst + regmap_offset,
                None => auto_offset,
            };
            // TODO should we force alignment when offset specified by user
            let mut sec_offset = align_on(bytes_align, raw_offset);

            // Check correctness of offset
            if sec_offset < auto_offset {
                return Err(RegmapError::Offset {
                    min_offset: auto_offset,
                    request_offset: sec_offset,
                    msg_info: format!("{:?}", section),
                }
                .into());
            }

            // Expand inner register
            let expanded_reg =
                Register::from_opt(&mut section.register.iter(), sec_offset, word_bytes)?;

            // Check range
            let real_range = expanded_reg
                .iter()
                .map(|reg| reg.offset + word_bytes)
                .max()
                .unwrap_or(sec_offset)
                - sec_offset;

            let range = if let Some(request_range) = section.range {
                if real_range > request_range {
                    return Err(RegmapError::Range {
                        request_range,
                        real_range,
                        msg_info: format!("{:?}", section),
                    }
                    .into());
                } else {
                    request_range
                }
            } else {
                real_range
            };

            // Handle duplication
            // -> No duplication is 1iteration without name extension
            for (i, s) in section
                .duplicate
                .clone()
                .unwrap_or(vec![String::new()])
                .iter()
                .enumerate()
            {
                // Patch offset if needed
                // NB: Have to regenerate register with updated offset in each duplicated section
                let register = if i != 0 {
                    sec_offset = align_on(bytes_align, sec_offset + range);
                    Register::from_opt(&mut section.register.iter(), sec_offset, word_bytes)?
                } else {
                    expanded_reg.clone()
                };

                let full_name = format!("{}{}", name, s);

                expanded_section.push(Self {
                    name: full_name,
                    description: section.description.clone(),
                    offset: sec_offset,
                    range,
                    bytes_align,
                    register,
                });
                // update auto_offset
                auto_offset = sec_offset + range;
            }
        }
        // Sort by offset
        expanded_section.sort_by(|a, b| a.offset.cmp(&b.offset));

        Ok(expanded_section)
    }
}

impl std::fmt::Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  name: {}", self.name)?;
        writeln!(f, "  description: {}", self.description)?;
        writeln!(f, "  offset: 0x{:x}", self.offset)?;
        writeln!(f, "  range:  0x{:x}", self.range)?;
        writeln!(f, "  bytes_align: {:?}", self.bytes_align)?;
        write!(f, "  Register: [")?;
        for reg in self.register.iter() {
            write!(f, "\n[{reg}]")?;
        }
        writeln!(f, "  ]")?;
        Ok(())
    }
}

#[derive(Debug, Getters, MutGetters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Regmap {
    #[getset(get_mut = "pub")]
    module_name: String,
    description: String,
    word_size_b: usize,
    offset: usize,
    range: usize,
    ext_pkg: HashSet<String>,
    section: Vec<Section>,
}

impl Regmap {
    pub fn from_opt(regmaps: &mut [parser::RegmapOpt]) -> Result<Self, anyhow::Error> {
        //1. Check compliance
        if regmaps.is_empty() {
            return Err(RegmapError::NoEntry.into());
        }
        // -> Couldn't merge regmap with != word_size_b
        let word_size_min = regmaps
            .iter()
            .map(|register| register.word_size_b)
            .min()
            .unwrap();
        let word_size_max = regmaps
            .iter()
            .map(|register| register.word_size_b)
            .max()
            .unwrap();
        let word_size_b = if word_size_min == word_size_max {
            word_size_min
        } else {
            return Err(RegmapError::WordSize.into());
        };

        //2. Order regmap slice based on their offset
        regmaps.sort_by(|a, b| a.offset.cmp(&b.offset));
        let global_offset = regmaps[0].offset.unwrap_or(0);

        //3. Fuse top-level properties
        let (module_name, description) = {
            let (name, descr) = regmaps
                .iter()
                .map(|r| (r.module_name.as_str(), r.description.as_str()))
                .collect::<(Vec<_>, Vec<_>)>();
            (
                name.as_slice().join("_").to_string(),
                descr.as_slice().join("\n\n").to_string(),
            )
        };
        let ext_pkg = regmaps
            .iter()
            .flat_map(|r| &r.ext_pkg)
            .cloned()
            .collect::<HashSet<_>>();

        //4. Expand regmap sections
        let mut global_section = Vec::new();
        let mut auto_offset = 0;
        let word_bytes = usize::div_ceil(word_size_b, u8::BITS as usize);

        for regmap in regmaps.iter() {
            // Compute offset and check correctness
            let regmap_offset = match regmap.offset {
                Some(ofst) => ofst,
                None => auto_offset,
            };
            if regmap_offset < auto_offset {
                return Err(RegmapError::Offset {
                    min_offset: auto_offset,
                    request_offset: regmap_offset,
                    msg_info: format!("{:?}", regmap),
                }
                .into());
            }
            // Check offset and range properties
            // Those properties is used to implement clever addr filtering in regmap module
            if !regmap.range.is_power_of_two() {
                return Err(RegmapError::RangePow2 {
                    range: regmap.range,
                    msg_info: format!("{:?}", regmap),
                }
                .into());
            }
            let range_mask = regmap.range - 1;
            if regmap_offset != (regmap_offset & !range_mask) {
                return Err(RegmapError::OffsetRange {
                    offset: regmap_offset,
                    range: regmap.range,
                    msg_info: format!("{:?}", regmap),
                }
                .into());
            }

            // Construct section
            let section = Section::from_opt(&mut regmap.section.iter(), regmap_offset, word_bytes)?;

            // Check range validity for the given regmap
            let real_range = section
                .iter()
                .map(|s| s.offset + s.range)
                .max()
                .unwrap_or(regmap_offset)
                - regmap_offset;
            if real_range > regmap.range {
                return Err(RegmapError::Range {
                    request_range: regmap.range,
                    real_range,
                    msg_info: format!("{:?}", regmap),
                }
                .into());
            }
            // Append section to global
            global_section.extend(section);

            // Update auto_offset for next iteration
            auto_offset = regmap_offset + regmap.range;
        }

        // Compute range of the regmap aggregation
        // It's computed as max_addr - global_offset
        let global_range = auto_offset - global_offset;

        Ok(Self {
            module_name,
            description,
            word_size_b,
            ext_pkg,
            offset: global_offset,
            range: global_range,
            section: global_section,
        })
    }
}

impl std::fmt::Display for Regmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "description: {}", self.description)?;
        writeln!(f, "  offset: 0x{:x}", self.offset)?;
        writeln!(f, "  range:  0x{:x}", self.range)?;
        writeln!(f, "  word_size_b: {:?}", self.word_size_b)?;
        writeln!(f, "External package: {:?}", self.ext_pkg)?;
        write!(f, "Section: [")?;
        for sec in self.section.iter() {
            write!(f, "\n[{sec}]")?;
        }
        writeln!(f, "]")?;
        Ok(())
    }
}
