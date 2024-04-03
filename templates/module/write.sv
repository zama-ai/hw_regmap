{# Template for generating storage and update logic #}          
{# Warn: Keep indentation in phase with module template (cf. ff_wr_snippets) #}          
{%- if not param_reg -%}
  // Register FF: {{name}}
  logic [REG_DATA_W-1:0] r_{{name}}D;

  {%  if reg_update %}
    {%  if wr_user %}
  assign r_{{name}}D = (wr_en && (wr_add == {{offset}}))? wr_data: r_{{name}}_upd;
    {% else %}
  assign r_{{name}}D       = r_{{name}}_upd;
    {% endif %}
  {% else %}
    {%  if wr_user %}
  assign r_{{name}}D = (wr_en && (wr_add == {{offset}}))? wr_data: r_{{name}};
    {% endif %}
  {% endif %}

  {% if wr_notify %}
  logic r_{{name}}_wr_enD;
  assign r_{{name}}_wr_enD = wr_en && (wr_add == {{ offset }});
  {% endif %}

  {% if wr_action %}
  logic [REG_DATA_W-1:0] r_{{name}}_wdataD;
  assign r_{{name}}_wdataD = wr_data;
  {% endif %}

  {% if rd_notify %}
  logic r_{{name}}_rd_enD;
  assign r_{{name}}_rd_enD = rd_en && (rd_add == {{ offset }});
  {% endif %}

  always_ff @(posedge clk) begin
    if (!s_rst_n) begin
      r_{{name}}       <= {{ default_val }};
      {% if wr_notify     %}r_{{name}}_wr_en <= 1'b0;         {% endif %}
      {% if wr_action     %}r_{{name}}_wdata <=  'h0;         {% endif %}
      {% if rd_notify     %}r_{{name}}_rd_en <= 1'b0;         {% endif %}
    end
    else begin
      r_{{name}}       <= r_{{name}}D;
      {% if wr_notify     %}r_{{name}}_wr_en <= r_{{name}}_wr_enD; {% endif %}
      {% if wr_action     %}r_{{name}}_wdata <= r_{{name}}_wdataD; {% endif %}
      {% if rd_notify     %}r_{{name}}_rd_en <= r_{{name}}_rd_enD; {% endif %}
    end
  end
{%- endif -%}
