    class {{section_name}}_section extends {{base_name}}_base_section;
        `uvm_object_utils({{section_name}}_section)
        function new(string name = "{{section_name}}");
            super.new(name);
        endfunction: new
    endclass: {{section_name}}_section
