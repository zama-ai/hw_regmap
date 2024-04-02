//!
//! Describe Register map abstract view
//! This is used for easy definition from the user, a lot of field are optionnals
//!
//! Also provide a set of function to serde it from/toward toml file
//!
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;

//NB: Owner, ReadAccess, WriteAccess are splitted to ease the Serde
//    and have a clear naming in toml without manual implementation of the serde traits
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum Owner {
    User,
    Kernel,
    Both,
    Parameter,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum ReadAccess {
    None,
    Read,
    ReadNotify,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum WriteAccess {
    None,
    Write,
    WriteNotify,
    WriteAction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FieldOpt {
    pub description: String,
    pub size_b: usize,
    pub offset_b: Option<usize>,
    pub default_val: Option<usize>,
    pub param_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterOpt {
    pub description: String,
    pub owner: Owner,
    pub read_access: ReadAccess,
    pub write_access: WriteAccess,
    pub default_val: Option<usize>,
    pub param_name: Option<String>,
    pub offset: Option<usize>,
    pub field: Option<IndexMap<String, FieldOpt>>,
    pub duplicate: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SectionOpt {
    pub description: String,
    pub offset: Option<usize>,
    pub align_offset: Option<bool>,
    pub duplicate: Option<Vec<String>>,
    pub register: IndexMap<String, RegisterOpt>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegmapOpt {
    pub description: String,
    pub word_size_b: usize,
    pub offset: Option<usize>,
    pub section: IndexMap<String, SectionOpt>,
}

impl RegmapOpt {
    pub fn read_from(file: &str) -> Self {
        let file_str = match fs::read_to_string(file) {
            Ok(str) => str,
            Err(err) => {
                panic!("Error: `{file}`:: {err}");
            }
        };

        match toml::from_str(&file_str) {
            Ok(regmap) => regmap,
            Err(err) => panic!("Error: {err}"),
        }
    }
}
