    class {{reg_name}}_reg extends uvm_reg;
        `uvm_object_utils({{reg_name}}_reg)
        {%- for  field in sv_fields%}
        rand uvm_reg_field {{field.name}};
        {%- endfor %}

        function new(string name = "{{reg_name}}");
            super.new(.name(name), .n_bit(32), .has_coverage(UVM_NO_COVERAGE))
        endfunction: new

        virtual function void build();
            {%- for  field in sv_fields%}
            {{field.name}} = uvm_reg_field::type_id::create("{{field.name}}");
            {{field.name}}.configure( .parent(this)
                                    , .size({{field.size}})
                                    , .lsb_pos({{field.lsb_pos}})
                                    , .access("{{field.access}}")
                                    , .volatile({{field.volatile}})
                                    , .reset({{field.reset}})
                                    , .has_reset({{field.has_reset}})
                                    , .is_rand({{field.is_rand}})
                                    , .individually_accessible({{field.individually_accessible}}));
        {%- endfor %}
        endfunction:build
    endclass: {{reg_name}}