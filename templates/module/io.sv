{# Template for generating RTL input/output #}
{# Warn: Keep indentation in phase with module template (cf. io_snippets) #}
    {%- if not param_reg %}  // Register IO: {{name}}{% endif %}
    {%  if not param_reg -%}
        {%  if have_fields -%}
        , output {{name}}_t r_{{name}}
        {% else -%}
        , output logic [REG_DATA_W-1: 0] r_{{name}}
        {% endif -%}
    {% endif -%}
    {%  if reg_update -%}
        {%  if have_fields -%}
        , input {{name}}_t r_{{name}}_upd
        {% else -%}
        , input  logic [REG_DATA_W-1: 0] r_{{name}}_upd
        {% endif -%}
    {% endif -%}
    {%  if rd_notify     %}, output logic r_{{name}}_rd_en                   {% endif %}
    {%  if wr_notify     %}, output logic r_{{name}}_wr_en                   {% endif %}
