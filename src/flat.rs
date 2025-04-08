//!
//! Provide a flatten view of register map
//! Useful on the Sw side to easily access register with offset and description

use getset::Getters;
use std::collections::HashMap;

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct FlatField {
    name: String,
    description: String,
    size_b: usize,
    offset_b: usize,
}
impl std::fmt::Display for FlatField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:-<40}", self.name)?;
        writeln!(f, "field: {}", self.description)?;
        writeln!(f, "size_b: {}", self.size_b)?;
        writeln!(f, "offset_b: {}", self.size_b)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum Access {
    None,
    Read,
    Write,
    ReadWrite,
}
impl std::fmt::Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let frag = match self {
            Access::None => "--",
            Access::Read => "Ro",
            Access::Write => "Wo",
            Access::ReadWrite => "RW",
        };
        write!(f, "{frag}")
    }
}

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct FlatRegister {
    // Section info
    sec_name: String,
    sec_description: String,
    // Reg info
    reg_name: String,
    reg_description: String,
    access: Access,
    offset: usize,
    // Field info
    field: Vec<FlatField>,
}
impl std::fmt::Display for FlatRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:-<80}", self.reg_name)?;
        writeln!(f, "section: {}", self.sec_name)?;
        writeln!(f, "section descr.: {}", self.sec_description)?;
        writeln!(f, "register descr.: {}", self.reg_description)?;
        writeln!(f, "access: {}", self.access)?;
        writeln!(f, "offset: 0x{:x}", self.offset)?;
        for field in self.field.iter() {
            write!(f, "{field}")?;
        }
        Ok(())
    }
}

impl FlatRegister {
    pub fn as_field(&self, value: u32) -> HashMap<String, u32> {
        self.field
            .iter()
            .map(
                |FlatField {
                     name,
                     size_b,
                     offset_b,
                     ..
                 }| {
                    let field_value = (value >> offset_b) & ((1 << size_b) - 1);
                    (name.clone(), field_value)
                },
            )
            .collect::<HashMap<_, _>>()
    }

    pub fn from_field(&self, field: HashMap<&str, u32>) -> u32 {
        let fields_map = self
            .field
            .iter()
            .map(
                |FlatField {
                     name,
                     size_b,
                     offset_b,
                     ..
                 }| (name, (size_b, offset_b)),
            )
            .collect::<HashMap<_, _>>();

        field
            .iter()
            .map(|(name, val)| {
                let (size_b, offset_b) = fields_map
                    .get(&name.to_string())
                    .unwrap_or_else(|| panic!("Field {name} isn't available in {:?}", self));
                (val & ((1 << *size_b) - 1)) << *offset_b
            })
            .sum()
    }
}

#[derive(Getters)]
#[getset(get = "pub")]
pub struct FlatRegmap {
    register: HashMap<String, FlatRegister>,
}
impl std::fmt::Display for FlatRegmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (hash_name, reg) in self.register.iter() {
            writeln!(f, "{:-<120}", hash_name)?;
            writeln!(f, "{reg}")?;
        }
        Ok(())
    }
}

impl FlatRegmap {
    pub fn from_file(regmap_toml: &[&str]) -> Self {
        // Parse regmap with optional fields
        let mut regmap_list = regmap_toml
            .iter()
            .map(|toml| crate::RegmapOpt::read_from(toml))
            .collect::<Vec<_>>();

        // Expand fields and check to have concrete regmap
        let regmap = crate::Regmap::from_opt(&mut regmap_list).unwrap();
        Self::new(regmap)
    }

    pub fn new(regmap: crate::Regmap) -> Self {
        let mut register = HashMap::new();
        regmap.section().iter().for_each(|sec| {
            sec.register().iter().for_each(|reg| {
                let hash_name = format!("{}::{}", sec.name(), reg.name());
                let field = if let Some(fmap) = reg.field() {
                    let mut field = Vec::new();
                    fmap.iter().for_each(|f| {
                        field.push(FlatField {
                            name: f.name().clone(),
                            description: f.description().clone(),
                            size_b: *f.size_b(),
                            offset_b: *f.offset_b(),
                        });
                    });
                    field
                } else {
                    vec![]
                };
                let access = match (reg.read_access().is_read(), reg.write_access().is_write()) {
                    (false, false) => Access::None,
                    (false, true) => Access::Write,
                    (true, false) => Access::Read,
                    (true, true) => Access::ReadWrite,
                };
                register.insert(
                    hash_name,
                    FlatRegister {
                        sec_name: sec.name().clone(),
                        sec_description: sec.description().clone(),
                        reg_name: reg.name().clone(),
                        reg_description: reg.description().clone(),
                        access,
                        offset: *reg.offset(),
                        field,
                    },
                );
            });
        });
        Self { register }
    }
}
