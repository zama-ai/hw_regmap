// ============================================================================================== //
// Description  : register  map address definition package
// This file was generated with rust regmap generator:
//  * Date:  {{ now() | date(format="%Y-%m-%d") }}
//  * Tool_version: {{ tool_version }}
// ---------------------------------------------------------------------------------------------- //
//
// Should only be used in testbench to drive the register interface
// ============================================================================================== //

package {{module_name}}_addr_pkg;
  {% for name, offset in regs_hash %}
  parameter int {{name}} = {{offset}};
  {% endfor %}
endpackage
