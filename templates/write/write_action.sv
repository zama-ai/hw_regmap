{# Template for generating storage and update logic #}          
{# Warn: Keep indentation in phase with module template (cf. ff_wr_snippets) #}          

  logic [REG_DATA_W-1:0] r_{{name}}D;
  logic [REG_DATA_W-1:0] r_{{name}}_wdataD;
  logic                  r_{{name}}_wr_enD;

  assign r_{{name}}D       = r_{{name}}_upd;
  assign r_{{name}}_wdataD = wr_data;

  assign r_{{name}}_wr_enD = wr_en && (wr_add == {{ offset }});

  always_ff @(posedge clk)
    if (!s_rst_n) begin
      r_{{name}}       <= {{ default }};
      r_{{name}}_wr_en <= 1'b0;
    end
    else begin
      r_{{name}}       <= r_{{name}}D;
      r_{{name}}_wr_en <= r_{{name}}_wr_enD;
    end

  always_ff @(posedge clk) begin
    r_{{name}}_wdata <= r_{{name}}_wdataD;
  end
