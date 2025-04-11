# HW Regmap

This repository contains a Hardware register map generation tool.
Based on a TOML definition, this utility generates:
* RTL module containing the registers in SystemVerilog
* Markdown documentation
* Runtime context for software definition

The main purpose is to ease the user experience on writing and using a register map.
For example, inference, respecting a set of properties, is applied when an option is not specified.
Property violations are reported by the tool with explicit and understandable error messages.


## RegisterMap definition
Register map is described with a hierarchical structure:
* Header
 * Unique
 * Define the register map main properties
* Section
 * Multiple
 * Organizational group of registers
* Register
 * Multiple within a section
 * Word that is accessible at an address
 * Define access rights, and characteristics
* Fields
 * Optional
 * Multiply within a register
 * Fields of the register word.
 * Define characteristics

### Header
Define the general properties of the register map. Some entries are optional and are automatically computed.
Available properties are:
* module_name: Name of the generated register map RTL module
* description: String describing the content of the register map
* word_size_b: Word size used in the register map (bit-unit)
* offset: Offset of the register map inside the global address map (byte-unit) [Optional][Default `0`]
* range: Range of addresses answered by the register map (byte-unit)
* ext_pkg: List of external SystemVerilog packages required by the register map RTL module (ex. package describing the AXI4-lite bus)

### Section
Registers are organized in sections. A section gathers sensible registers together, at a given address offset.
Available properties are:
* description: String describing the content of the section
* offset: Offset of the section in the register map (byte-unit) [Optional][Default `automatic`]
* range: Range of addresses covered by the section (byte-unit) [Optional][Default `automatic`]
* bytes_align: Required address alignment for the section (byte-unit) [Optional][Default `automatic`]
* duplicate: Multiple instances with same set of registers. The argument is a list of suffix to be applied on the section name [Optional][Default `None`]


### Register
Fine control of register read/write access is possible. Corresponding SystemVerilog code is implemented.
Available properties are:
* description: String describing the content of the register
* owner: Entity that handles physical register update. Available options are [User, Kernel, Parameter].
* read_access: Read access properties: availability, HW notification if reading.
               Available options [None, Read, ReadNotify]
* write_access: Write access properties: availability, HW notification if writing.
               Available options [None, Write, WriteNotify]
* default: Default value at reset. If the register stores a constant, the format is {Cst=<val>}. If the value comes from a systemVerilog parameter, the format is {Param="<param_name>"}. Note that if not used, the default value is 0. [Optional][Default `{Cst=0}`]
* bytes_align: Required address alignment for the register (byte-unit) [Optional][Default `automatic`]
* offset: Offset of the register in the section (byte-unit) [Optional][Default `automatic`]
* duplicate: Multiple instances of this register. The argument is a list of suffix to be applied on the register name [Optional][Default None]

Example 1: register exposing a RTL parameter to the user.
* This register is read only.
* It gets its value from a RTL systemVerilog parameter VERSION.
``` toml 
[section.rtl_properties.register.version]
  description="Version of the current hardware"
  owner="Parameter"
  read_access="Read"
  write_access="None"
  default={Param="VERSION"}
```

Example 2: register storing runtime configuration defined by the user.
* This register is a read / write register.
* On write action a notification is sent to the HW. Thus the module could handle any necessary action on this trigger.
* The default value on reset is 0x0000FFFF.
``` toml 
[section.rtl_properties.register.timeout_cnt]
  description="Wrapping value of the timeout counter"
  owner="User"
  read_access="Read"
  write_access="WriteNotify"
  default={Cst=0x0000FFFF}
```

Example 3: register exposing performance counter.
* This register is read only.
* Its value is updated by the RTL.
* On Read action a notification is sent to the HW, which would do the necessary to reset the register value.
``` toml 
[section.rtl_properties.register.perf_cnt]
  description="Performance counter. Reset on Read"
  owner="Kernel"
  read_access="ReadNotify"
  write_access="None"
```

### Fields
A field is an optional property of registers.
A Register can be composed of several fields. A set of functions is available to retrieve/update registers with a field aware method.
A field has a name.
Available field properties are:
* size_b: Number of bits used by the field (bit-unit)
* offset_b: Offset within the register word (bit-unit) [Optional][Default `automatic`]
* default: Specify default value after a reset. Could use a constant value or a RTL parameter. (same syntax as register default property) [Optional][Default `{Cst=0}`]

Example: register describing the HW version, seen as composed by 3 fields:
``` toml 
[section.rtl_properties.register.version]
  description="Version of the current HW"
  owner="Parameter"
  read_access="Read"
  write_access="None"
  field.vendor_id = { size_b=16, offset_b=0 ,  default={Cst=0xdead}, description="Vendor Id"}
  field.major     = { size_b=8, offset_b=16 , default={Param="MAJOR_REV"}, description="Major version number"}
  field.minor     = { size_b=8, offset_b=24 , default={Param="MINOR_REV"}, description="Minor version number"}
```

## SystemVerilog registers
To generate RTL sources, the TOML register map is parsed by the tool. Missing optional fields are computed. The defined and inferred values are checked in compliance with a set of properties.
A concrete register map is then built in memory and a set of [Tera](https://github.com/Keats/tera) templates are used to convert it in a SystemVerilog description.
The set of provided Tera templates can be easily edited by the user to adapt the generated construct to specific application needs.

## Runtime context
This repository could be used as an external library. It enables software to digest the register map definition and provides a flat-map view of it for easy `Register` to `Address` translation.
This way, the same TOML description can be used for RTL generation and inside the SW driver.

From a SW perspective, the TOML definition can be parsed in a flat hash table that enables the SW to get register content from a name.

``` rust
// ~~ ---
let reg = regmap
    .register()
    .get("rtl_properties::version")
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
Example picturing available syntax flavors. A monolithic register map is generated.
``` bash
cargo run -- --output-path gen --toml-file config/example.toml
```

### Debug offset 
Example playing with offset features. Offset can be fixed or computed by the tool.
``` bash
cargo run -- --output-path gen --toml-file config/debug/offset.toml
```

### Multi-regmap
Example demonstrating the multi-regmap capability. Register map can be split into multiple RTL modules to ease physical place and route.
The tool enforces the overall coherency of the generated addresses while generating multiple RTL modules.
``` bash
cargo run -- --output-path gen --toml-file config/debug/many/slice_a.toml --toml-file config/debug/many/slice_b.toml
```
