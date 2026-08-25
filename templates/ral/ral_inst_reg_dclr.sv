  class {{reg_name}}_reg extends {{base_reg_name}}_reg;
      `uvm_object_utils({{reg_name}}_reg)
      function new(string name = "{{reg_name}}");
          super.new(name);
      endfunction: new
  endclass: {{reg_name}}_reg
