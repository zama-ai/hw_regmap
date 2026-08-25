    class {{base_name}}_base_section extends uvm_reg_block;
    {%- for reg in registers%}
        rand {{base_name}}_{{reg.name}}_base_reg {{reg.name}};
    {%- endfor %}
        `uvm_object_utils({{base_name}}_base_section)
        function new(string name = "{{base_name}}_base");
          super.new(name);
        endfunction: new
        virtual function void build();
          this.default_map = create_map(.name("default_map"),
                                        .base_addr(`UVM_REG_ADDR_WIDTH'h0),
                                        .n_bytes(4),
                                        .endian(UVM_LITTLE_ENDIAN),
                                        .byte_addressing(1));
          {%- for reg in registers%}
          this.{{reg.name}} = {{base_name}}_{{reg.name}}_base_reg::type_id::create("{{reg.name}}");
          this.{{reg.name}}.configure(.blk_parent(this), .regfile_parent(null), .hdl_path(""));
          this.{{reg.name}}.build();
          this.default_map.add_reg({{reg.name}}, `UVM_REG_ADDR_WIDTH{{as_sv_hex(val=reg.relative_offset)}}, "RW");
          {%- endfor %}
          this.lock_model();
        endfunction:build
    endclass:{{base_name}}_base_section
