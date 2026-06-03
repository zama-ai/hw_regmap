  class {{reg_name}}_reg extends uvm_reg;
      `uvm_object_utils({{reg_name}}_reg)
      {%- for  field in sv_fields%}
      rand uvm_reg_field {{field.name}};
      {%- endfor %}

      function new(string name = "{{reg_name}}");
          super.new(.name(name), .n_bits(32), .has_coverage(UVM_NO_COVERAGE));
      endfunction: new

      virtual function void build();
      {%- for  field in sv_fields%}
          this.{{field.name}} = uvm_reg_field::type_id::create("{{field.name}}");
          this.{{field.name}}.configure( .parent(this)
                                       , .size({{field.size}})
                                       , .lsb_pos({{field.lsb_pos}})
                                       , .access("{{field.access}}")
                                       {%- for type,val in field.reset -%}
                                       {%- if type is containing("Param")%}{# TODO #}
                                       , .volatile(1)
                                       , .reset('h0)
                                       , .has_reset(1)
                                       {%- elif type is containing("Cst") %}{# Raw constant format as system_verilog hex #}
                                       , .volatile(0)
                                       , .reset({{as_sv_hex(val=val)}})
                                       , .has_reset(1)
                                       {%- else %}
                                       , .volatile(1)
                                       , .reset('h0)
                                       , .has_reset(0)
                                       {%- endif -%}
                                       {%- endfor %}
                                       , .is_rand({{field.is_rand}})
                                       , .individually_accessible({{field.individually_accessible}}));
      {%- endfor %}
      endfunction:build
  endclass: {{reg_name}}_reg