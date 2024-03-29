{# Template for generating storage and update logic #}          
{# Warn: Keep indentation in phase with module template (cf. ff_wr_snippets) #}          
  logic [REG_DATA_W-1:0] r_{{name}}D;

  always_ff @(posedge clk)
    if (!s_rst_n) begin
      r_{{name}} <= {{ default }};
    end
    else begin
      r_{{name}} <= r_{{name}}D;
    end

  assign r_{{name}}D = (wr_en && (wr_add == {{offset}}))? wr_data: r_{{name}};
