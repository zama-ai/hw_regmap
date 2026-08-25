package {{module_name}}_ral_pkg;

  import uvm_pkg::*;
  `include "uvm_macros.svh"

{%- for reg in ral_regs%}
  typedef class {{reg.name}}_reg;
{%- endfor %}
{%- for sct in ral_scts%}
  typedef class {{sct.name}}_section;
{%- endfor %}
{%- for group in ral_dup_groups%}
  typedef class {{group.base_name}}_base_section;
{%- for reg_name in group.base_reg_names%}
  typedef class {{reg_name}}_reg;
{%- endfor %}
{%- for inst in group.instances%}
  typedef class {{inst.name}}_section;
{%- endfor %}
{%- for reg_name in group.inst_reg_names%}
  typedef class {{reg_name}}_reg;
{%- endfor %}
{%- endfor %}
  typedef class {{module_name}}_block;

{% for reg in ral_regs -%}
{{reg.regd_snippet}}
{% endfor %}

{%- for sct in ral_scts%}
{{sct.sctd_snippet}}
{%- endfor %}

{%- for group in ral_dup_groups%}
{%- for snip in group.base_reg_snippets%}
{{snip}}
{%- endfor %}
{%- for snip in group.inst_reg_snippets%}
{{snip}}
{%- endfor %}
{{group.base_sct_snippet}}
{%- for inst in group.instances%}
{{inst.sctd_snippet}}
{%- endfor %}
{%- endfor %}

    class {{module_name}}_block extends uvm_reg_block;
    {%- for sct in ral_scts%}
        rand {{sct.name}}_section {{sct.name}};
    {%- endfor %}
    {%- for group in ral_dup_groups%}
    {%- for inst in group.instances%}
        rand {{inst.name}}_section {{inst.name}};
    {%- endfor %}
        {{group.base_name}}_base_section {{group.base_name}}_l[{{group.instances | length}}];
    {%- for fname in group.reg_field_names%}
        {{group.base_name}}_{{fname}}_base_reg {{group.base_name}}_{{fname}}_l[{{group.instances | length}}];
    {%- endfor %}
    {%- endfor %}
        `uvm_object_utils({{module_name}}_block)
        function new(string name = "module_reg");
            super.new(name);
        endfunction: new
        virtual function void build();
            this.default_map = create_map(.name("default_map"),
                                     .base_addr(`UVM_REG_ADDR_WIDTH'h0),
                                     .n_bytes(4),
                                     .endian(UVM_LITTLE_ENDIAN),
                                     .byte_addressing(1));
            {%- for sct in ral_scts%}
            this.{{sct.name}} = {{sct.name}}_section::type_id::create("{{sct.name}}");
            this.{{sct.name}}.configure(.parent(this));
            this.{{sct.name}}.build();
            this.default_map.add_submap({{sct.name}}.default_map, `UVM_REG_ADDR_WIDTH'h{{sct.offset}});
            {%- endfor %}
            {%- for group in ral_dup_groups%}
            {%- for inst in group.instances%}
            this.{{inst.name}} = {{inst.name}}_section::type_id::create("{{inst.name}}");
            this.{{inst.name}}.configure(.parent(this));
            this.{{inst.name}}.build();
            this.default_map.add_submap({{inst.name}}.default_map, `UVM_REG_ADDR_WIDTH'h{{inst.offset}});
            {%- endfor %}
            {%- for inst in group.instances%}
            this.{{group.base_name}}_l[{{loop.index0}}] = this.{{inst.name}};
            {%- endfor %}
            {%- for fname in group.reg_field_names%}
            {%- for inst in group.instances%}
            this.{{group.base_name}}_{{fname}}_l[{{loop.index0}}] = this.{{inst.name}}.{{fname}};
            {%- endfor %}
            {%- endfor %}
            {%- endfor %}
            this.lock_model();
        endfunction:build
    endclass:{{module_name}}_block
endpackage:{{module_name}}_ral_pkg
