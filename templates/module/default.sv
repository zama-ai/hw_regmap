{# Template for generating default signals #}
{# Warn: Keep indentation in phase with module template (cf. default_snippets) #}

{# Not a map, but it's the way to match on enum in tera #} 
{%for type,val in default_val %}
//-- Default {{name}}
{%if type is containing("ParamsField") %}
  {{name}}_t {{name}}_default;
  always_comb begin
    {{name}}_default = '0;
    {%for nv in val.name_val %}
    {{name}}_default.{{nv.0}} = {{nv.1}};
    {%endfor%}
  end
{%else%}
  logic [REG_DATA_W-1:0]{{name}}_default;
  assign {{name}}_default = {{val}};
{%endif%}
{%endfor%}
