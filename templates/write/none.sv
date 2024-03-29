{# Template for generating storage and update logic #}          
{# Warn: Keep indentation in phase with module template (cf. ff_wr_snippets) #}          

  logic [REG_DATA_W-1:0] r_{{name}}D;

  assign r_{{name}}D = r_{{name}}_upd;

  always_ff @(posedge clk)
    if (!s_rst_n) begin
      r_{{name}} <= {{ default }};
    end
    else begin
      r_{{name}} <= r_{{name}}D;
    end
