// ============================================================================================== //
// Description  : Axi4-lite register bank
// This file was generated with rust regmap generator:
//  * Date:  {{ now() | date(format="%Y-%m-%d") }}
//  * Tool_version: {{ tool_version }}
// ---------------------------------------------------------------------------------------------- //
// xR[n]W[na]
// |-> who is in charge of the register update logic : u -> User
//                                                   : k -> Kernel (with an *_upd signal)
//                                                   : p -> Parameters (i.e. constant register)
//  | Read options
//  | [n] optional generate read notification (have a _rd_en)
//  | Write options
//  | [n] optional generate wr notification (have a _wr_en)
// 
// Thus type of registers are:
// uRW  : Read-write                                              
//      : Value provided by the host. The host can read it and write it.
// uW   : Write-only                                              
//      : Value provided by the host. The host can only write it.
// uWn  : Write-only with notification                            
//      : Value provided by the host. The host can only write it.
// kR   : Read-only register                                      
//      : Value provided by the RTL.
// kRn  : Read-only register with notification  (rd)              
//      : Value provided by the RTL.
// kRWn : Read-only register with notification (wr)
//      : Value provided by the RTL. The host can read it. The write data is processed by the RTL.
// kRnWn: Read-only register with notification (rd/wr)
//      : Value provided by the RTL. The host can read it with notify. The write data is processed by the RTL.
// ============================================================================================== //

module {{module_name}}
{%for pkg in ext_pkg%}
import {{pkg}}::*;
{%endfor%}
import {{module_name}}_pkg::*;
{% raw %}#({% endraw %}
  {%- set_global put_comma = 0 -%}
  {%- for reg in regs_sv -%}
  {%- if reg.param_snippets != "" -%}
  {%- if put_comma > 0 %},{% endif -%}
  {{reg.param_snippets}}
  {%- set_global put_comma = 1 -%}
  {%- endif -%}
  {%- endfor -%})(
  input  logic                           clk,
  input  logic                           a_rst_n,

  // Axi4 lite Slave Interface sAxi4
  input  logic [AXIL_ADD_W-1:0]         s_axil_awaddr,
  input  logic                          s_axil_awvalid,
  output logic                          s_axil_awready,
  input  logic [AXIL_DATA_W-1:0]        s_axil_wdata,
  input  logic                          s_axil_wvalid,
  output logic                          s_axil_wready,
  output logic [AXI4_RESP_W-1:0]        s_axil_bresp,
  output logic                          s_axil_bvalid,
  input  logic                          s_axil_bready,
  input  logic [AXIL_ADD_W-1:0]         s_axil_araddr,
  input  logic                          s_axil_arvalid,
  output logic                          s_axil_arready,
  output logic [AXIL_DATA_W-1:0]        s_axil_rdata,
  output logic [AXI4_RESP_W-1:0]        s_axil_rresp,
  output logic                          s_axil_rvalid,
  input  logic                          s_axil_rready,
  // Registered version of wdata
  output logic [AXIL_DATA_W-1:0]        r_axil_wdata

  {%- for reg in regs_sv -%}
  {%- if reg.io_snippets != "" -%}{{reg.io_snippets}}{%- endif -%}
  {%- endfor -%}
);

// ============================================================================================== --
// localparam
// ============================================================================================== --
  localparam int AXIL_ADD_OFS = {{as_sv_hex(val=offset)}};
  localparam int AXIL_ADD_RANGE= {{as_sv_hex(val=range)}}; // Should be a power of 2

  localparam int AXIL_ADD_RANGE_W = $clog2(AXIL_ADD_RANGE);
  localparam [AXIL_ADD_W-1:0] AXIL_ADD_RANGE_MASK = AXIL_ADD_W'(AXIL_ADD_RANGE - 1);
  localparam [AXIL_ADD_W-1:0] AXIL_ADD_OFS_MASK   = ~(AXIL_ADD_W'(AXIL_ADD_RANGE - 1));

// ============================================================================================== --
// axil management
// ============================================================================================== --
  logic                    axil_awready;
  logic                    axil_wready;
  logic [AXI4_RESP_W-1:0]  axil_bresp;
  logic                    axil_bvalid;
  logic                    axil_arready;
  logic [AXI4_RESP_W-1:0]  axil_rresp;
  logic [AXIL_DATA_W-1:0]  axil_rdata;
  logic                    axil_rvalid;

  logic                    axil_awreadyD;
  logic                    axil_wreadyD;
  logic [AXI4_RESP_W-1:0]  axil_brespD;
  logic                    axil_bvalidD;
  logic                    axil_arreadyD;
  logic [AXI4_RESP_W-1:0]  axil_rrespD;
  logic [AXIL_DATA_W-1:0]  axil_rdataD;
  logic                    axil_rvalidD;

  logic                    wr_en;
  logic [AXIL_ADD_W-1:0]   wr_add;
  logic [AXIL_DATA_W-1:0]  wr_data;
  logic                    rd_en;
  logic [AXIL_ADD_W-1:0]   rd_add;

  logic                    wr_enD;
  logic [AXIL_ADD_W-1:0]   wr_addD;
  logic [AXIL_DATA_W-1:0]  wr_dataD;
  logic                    rd_enD;
  logic [AXIL_ADD_W-1:0]   rd_addD;

  logic                    wr_en_okD;
  logic                    rd_en_okD;
  logic                    wr_en_ok;
  logic                    rd_en_ok;

  //== Check address
  // Answer all requests within [ADD_OFS -> ADD_OFS + RANGE[
  // Since RANGE is a power of 2, this could be done with masks.
  logic s_axil_wr_add_ok;
  logic s_axil_rd_add_ok;

  assign s_axil_wr_add_ok = (s_axil_awaddr & AXIL_ADD_OFS_MASK) == AXIL_ADD_OFS;
  assign s_axil_rd_add_ok = (s_axil_araddr & AXIL_ADD_OFS_MASK) == AXIL_ADD_OFS;

  //== Local read/write signals
  // Write when address and data are available.
  // Do not accept a new write request when the response
  // of previous request is still pending.
  // Since the ready is sent 1 cycle after the valid,
  // mask the cycle when the ready is r
  assign wr_enD   = (s_axil_awvalid & s_axil_wvalid
                     & ~(s_axil_awready | s_axil_wready)
                     & ~(s_axil_bvalid & ~s_axil_bready));
  assign wr_en_okD = wr_enD & s_axil_wr_add_ok;
  assign wr_addD  = s_axil_awaddr;
  assign wr_dataD = s_axil_wdata;

  // Answer to read request 1 cycle after, when there is no pending read data.
  // Therefore, mask the rd_en during the 2nd cycle.
  assign rd_enD   = (s_axil_arvalid
                    & ~s_axil_arready
                    & ~(s_axil_rvalid & ~s_axil_rready));
  assign rd_en_okD = rd_enD & s_axil_rd_add_ok;
  assign rd_addD   = s_axil_araddr;

  //== AXIL write ready
  assign axil_awreadyD = wr_enD;
  assign axil_wreadyD  = wr_enD;

  //== AXIL read address ready
  assign axil_arreadyD = rd_enD;

  //== AXIL write resp
  assign axil_bvalidD    = wr_en         ? 1'b1:
                           s_axil_bready ? 1'b0 : axil_bvalid;
  assign axil_brespD     = wr_en         ? wr_en_ok ? AXI4_OKAY : AXI4_SLVERR:
                           s_axil_bready ? 1'b0 : axil_bresp;

  //== AXIL read resp
  assign axil_rvalidD    = rd_en         ? 1'b1 :
                           s_axil_rready ? 1'b0 : axil_rvalid;

  `ALWAYS_FF(clk, a_rst_n) begin
    if (!a_rst_n) begin
      axil_awready <= 1'b0;
      axil_wready  <= 1'b0;
      axil_bresp   <= '0;
      axil_bvalid  <= 1'b0;

      axil_arready <= 1'b0;
      axil_rdata   <= '0;
      axil_rresp   <= '0;
      axil_rvalid  <= 1'b0;

      wr_en        <= 1'b0;
      rd_en        <= 1'b0;

      wr_en_ok     <= 1'b0;
      rd_en_ok     <= 1'b0;
    end
    else begin
      axil_awready <= axil_awreadyD;
      axil_wready  <= axil_wreadyD;
      axil_bresp   <= axil_brespD;
      axil_bvalid  <= axil_bvalidD;

      axil_arready <= axil_arreadyD;
      axil_rdata   <= axil_rdataD;
      axil_rresp   <= axil_rrespD;
      axil_rvalid  <= axil_rvalidD;

      wr_en         <= wr_enD;
      rd_en         <= rd_enD;

      wr_en_ok      <= wr_en_okD;
      rd_en_ok      <= rd_en_okD;
    end
  end

  always_ff @(posedge clk) begin
    wr_add  <= wr_addD;
    rd_add  <= rd_addD;
    wr_data <= wr_dataD;
  end

  //= Assignment
  assign s_axil_awready = axil_awready;
  assign s_axil_wready  = axil_wready;
  assign s_axil_bresp   = axil_bresp;
  assign s_axil_bvalid  = axil_bvalid;
  assign s_axil_arready = axil_arready;
  assign s_axil_rresp   = axil_rresp;
  assign s_axil_rdata   = axil_rdata;
  assign s_axil_rvalid  = axil_rvalid;
  assign r_axil_wdata   = wr_data;

// ============================================================================================== --
// Default value signals
// ============================================================================================== --
{%- for reg in regs_sv -%}{{reg.default_snippets}}{% endfor %}

// ============================================================================================== --
// Write reg
// ============================================================================================== --
  // To ease the code, use REG_DATA_W as register size.
  // Unused bits will be simplified by the synthesizer
  {%- for reg in regs_sv -%}{{reg.ff_wr_snippets}}{% endfor %}

// ============================================================================================== --
// Read reg
// ============================================================================================== --
  always_comb begin
    if (axil_rvalid) begin
      axil_rdataD = s_axil_rready ? '0 : axil_rdata;
      axil_rrespD = s_axil_rready ? '0 : axil_rresp;
    end
    else begin
      axil_rdataD = axil_rdata;
      axil_rrespD = axil_rresp;
      if (rd_en) begin
        if (!rd_en_ok) begin
          axil_rdataD = REG_DATA_W'('hDEAD_ADD2);
          axil_rrespD = AXI4_SLVERR;
        end
        else begin
          axil_rrespD = AXI4_OKAY;
          case(rd_add[AXIL_ADD_RANGE_W-1:0])
          {%- for reg in regs_sv -%}{{reg.rd_snippets}}{% endfor %}
          default:
            axil_rdataD = REG_DATA_W'('h0BAD_ADD1); // Default value
          endcase // rd_add
        end
      end // if rd_end
    end
  end // always_comb - read

endmodule
