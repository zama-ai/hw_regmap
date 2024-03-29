{# Template for generating storage and update logic #}          
{# Warn: Keep indentation in phase with module template (cf. ff_wr_snippets) #}          
  logic [REG_DATA_W-1:0] r_{{name}}D;
  logic                  r_{{name}}_wr_enD;

  always_ff @(posedge clk)
    if (!s_rst_n) begin
      r_{{name}}       <= {{ default }};
      r_{{name}}_wr_en <= 1b'0;
    end
    else begin
      r_{{name}}       <= r_{{name}}D;
      r_{{name}}_wr_en <= r_{{name}}_wr_enD;
    end

  always_comb begin
    r_{{name}}D       = r_{{name}}_upd;
    r_{{name}}_wr_enD = 1'b0;
    if (wr_en) begin
      case (wr_add)
        {{ offset }}: begin
          r_{{name}}D       = wr_data;
          r_{{name}}_wr_enD = 1'b1;
        end
      endcase
    end // if wr_en
  end // always_comb - write
