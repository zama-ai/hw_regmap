package {{module_name}}_ral_pkg;

{%- for reg in ral_regs%}
typedef class {{reg.name}}_reg;
{%- endfor %}
{%- for  sct in ral_scts%}
typedef class {{sct.name}}_section;
{%- endfor %}
typedef class {{module_name}}_block;

{% for reg in ral_regs -%}
{{reg.regd_snippet}}
{%- endfor %}

{%- for sct in ral_scts%}
{{sct.sctd_snippet}}
{%- endfor %}
    class {{module_name}}_block extends uvm_reg_block;
    {%- for  sct in ral_scts%}
        rand {{sct.name}}_section {{sct.name}};
    {%- endfor %}
        `uvm_object_utils({{module_name}}_section)
        function new(string name = "module_reg");
            super.new(name);
        endfunction: new
        virtual function void build();
            default_map = create_map(.name("default_map"),
                                     .base_addr(`UVM_REG_ADDR_WIDTH'h0),
                                     .n_bytes(4),
                                     .endian(UVM_LITTLE_ENDIAN),
                                     .byte_addressing(1));
            {%- for  sct in ral_scts%}
            {{sct.name}} = {{sct.name}}_section::type_id::create("{{sct.name}}");
            {{sct.name}}.configure(.blk_parent(this));
            {{sct.name}}.build();
            this.default_map.add_submap({{sct.name}}.default_map, `UVM_REG_ADDR_WIDTH'h{{sct.offset}});
            {%- endfor %}
        endfunction:build
    endclass:{{module_name}}_block
endpackage:{{module_name}}_ral_pkg