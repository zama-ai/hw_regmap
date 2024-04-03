{# Template for generating RTL input/output #}
{# Warn: Keep indentation in phase with module template (cf. io_snippets) #}
    {%- if not param_reg %}  // Register IO: {{name}}                                {% endif %}
    {%  if not param_reg %}, output logic [REG_DATA_W-1: 0] r_{{name}}       {% endif %}
    {%  if reg_update    %}, input  logic [REG_DATA_W-1: 0] r_{{name}}_upd   {% endif %}
    {%  if wr_action     %}, output logic [REG_DATA_W-1: 0] r_{{name}}_wdata {% endif %}
    {%  if rd_notify     %}, output logic r_{{name}}_rd_en                   {% endif %}
    {%  if wr_notify     %}, output logic r_{{name}}_wr_en                   {% endif %}
