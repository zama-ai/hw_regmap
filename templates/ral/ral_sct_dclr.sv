    class {{section_name}}_section extends uvm_reg_block;
    {%- for  reg in registers%}
        rand {{reg.name}}_reg {{reg.name}};
    {%- endfor %}
        `uvm_object_utils({{section_name}}_section)
        function new(string name = "{section_name}}");
            super.new(name);
        endfunction: new
        virtual function void build();
            default_map = create_map(.name("default_map"),
                                     .base_addr(`UVM_REG_ADDR_WIDTH'h0),
                                     .n_bytes(4),
                                     .endian(UVM_LITTLE_ENDIAN),
                                     .byte_addressing(1));
            {%- for  reg in registers%}
            {{reg.name}} = {{reg.name}}_reg::type_id::create("{{reg.name}}");
            {{reg.name}}.configure(.blk_parent(this), .regfile_parent(null), .hdl_path(""));
            {{reg.name}}.build();
            this.default_map.add_reg({{reg.name}}, `UVM_REG_ADDR_WIDTH'h{{reg.offset}}, "RW");
            {%- endfor %}
        endfunction:build
    endclass:{{section_name}}_section