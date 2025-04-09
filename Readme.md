# HW Regmap

This repository contains a Hardware register map generation tool.
Based on a TOML definition, this utility could generate:
* SystemVerilog registers
* Markdown documentation
* Runtime context for SW definition

Objectives are to relax requirements on the user side. When options are not specified the tool infers them while maintaining a set of properties.
Error in the description or property violations (e.g. Defined register that cannot fit in the allocated range, etc.) are reported by the tool with explicit and understandable error messages.


## RegisterMap definition
Register map is described with a hierarchical structure:
* Top-level properties
* Sections: Gather several related registers in a group
* Register: Define access right of a given register map word
* Fields: Split a register in sub-fields that can be accessed/updated with dedicated setter/getter.

### RegisterMap
Define the top-level properties of the register map. Some fields are optional and are automatically computed.
Available properties are:
* module_name: Name of the generated register map module
* description: String describing the content of the register map
* word_size_b: Word size used in the register map
* offset: Offset of the register map inside global address map.
* range: Range of address answered by the register map
* ext_pkg: External SystemVerilog package required by the register map (i.e. used by constant register exposing RTL parameters to the user)

Section can use `duplicate` keyword to create multiple instance with same set of registers

### Section
Users can gather a set of registers in a given section. Enables gathering meaningful registers together and putting them at a given offset.
Available properties are:
* description: String describing the content of the section
* offset: Offset of the section in the register map

### Register
Registers are defined with a set of access rights that enable fine grain control of the accessibility.
It's used to select the underlying SystemVerilog template and enforce properties such as Read/Write access.
Available properties are:
* description: String describing the content of the section
* owner: Depicts entity that handles register update. Available options are [User, Kernel, Parameter].
* read_access: Can this register be read from the interface, does it trigger notifications.
               Available options [None, Read, ReadNotify]
* write_access: Can this register be written from the interface, does it triggered notifications
               Available options [None, Write, WriteNotify]

Below is an example of a register used to expose an RTL parameter to the user. This register will get its value from an RTL parameter (defined as parameter in the SystemVerilog module) and could only be Read from the user perspective.
``` toml 
[section.RtlProperties.register.Version]
  description="Version of the current HW"
  owner="Parameter"
  read_access="Read"
  write_access="None"
```

Below is an example of a register used to retrieve runtime configuration from the user. It can be read and written from a user perspective. On write, the RTL is notified to handle internal update
``` toml 
[section.RtlProperties.register.Version]
  description="Wrapping value of the timeout counter"
  owner="User"
  read_access="Read"
  write_access="WriteNotify"
```

Below is a example of a register used to expose performance value to the user.
It is updated by the RTL and reset upon read by the user.
``` toml 
[section.RtlProperties.register.Version]
  description="Performance counter. Reset on Read"
  owner="Kernel"
  read_access="ReadNotify"
  write_access="None"
```

Register can use the `duplicate` keyword to create multiple instances with the same properties.

### Fields
Registers can be composed of sub-fields. A set of functions is available to retrieve/update registers with a field aware method.
Available properties are:
* name: Name of the field
* size_b: Number of bits used by the field
* offset_b: Offset in bits within the register word
* default: Specify default value after a reset. Could use a constant value or an RTL parameters

Below a version register content is divided in 3 fields:
``` toml 
[section.RtlProperties.register.Version]
  description="Version of the current HW"
  owner="Parameter"
  read_access="Read"
  write_access="None"
  field.vendor_id = { size_b=16, offset_b=0 ,  default={Cst=0xdead}, description="Vendor Id"}
  field.major     = { size_b=8, offset_b=16 , default={Param="MAJOR_REV"}, description="Major version number"}
  field.minor     = { size_b=8, offset_b=24 , default={Param="MINOR_REV"}, description="Minor version number"}
```

## SystemVerilog registers
To generate RTL sources, the TOML register map is parsed, missing optional fields are computed and a set of properties is checked.
A concrete register map is then built in memory and a set of [Tera](https://github.com/Keats/tera) templates are used to convert it in a SystemVerilog description.
The set of provided Tera templates can be easily edited by the user to adapt the generated construct to specific application needs.

## Runtime context
This repository could be used as an external library. It enables Software to digest the register map definition and provide a flat-map view of it for easy `Register` to `Address` translation.
This way, the same TOML description can be used for RTL generation and inside SW driver.

From an SW perspective, the TOML definition can be parsed in a flat hash table that enables SW to get register content from a name.

``` rust
// ~~ ---
let reg = regmap
    .register()
    .get("RtlProperties::Version")
    .expect("Unknown register, check regmap definition");
let val = ffi_hw.read_reg(*reg.offset() as u64);
let fields = reg.as_field(val);
let vendor_id = *fields.get("vendor_id").expect("Unknown field");
let major_version = *fields.get("major").expect("Unknown field");
let minor_version = *fields.get("minor").expect("Unknown field");
// ~~ ---
```

## Examples
The config folder contains some examples that show register map capabilities.

### Base
Simple example that shows available syntax flavors. It generates a monolithic register map.
``` bash
cargo run -- --output-path gen --toml-file config/example.toml
```

### Debug offset 
Simple example that depicts offset features. Offset can be fixed or computed by the tool.
``` bash
cargo run -- --output-path gen --toml-file config/debug/offset.toml
```

### Multi-regmap
Simple example that depicts the multi-regmap capabilities. Register map can be generated in multiple modules to ease RTL place and route.
The tool enforces the overall coherency of the generated addresses while generating multiple RTL modules.
``` bash
cargo run -- --output-path gen --toml-file config/debug/many/slice_a.toml --toml-file config/debug/many/slice_b.toml
```
