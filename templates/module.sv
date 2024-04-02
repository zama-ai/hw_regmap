// ============================================================================================== //
// Description  : Axi4-lite register bank
// This file was generated with rust regmap generator:
//  * Date:  {{ now }}
// ---------------------------------------------------------------------------------------------- //
// TODO update naming definition
// xR[n]W[na]
// |-> who is in charge of the register update logic : u -> User
//                                                   : k -> Kernel (have a _upd signal)
//                                                   : b -> Both
//  | Read options
//  | [n] optional generate read notification (have a _rd_en)
//      | Write options
//      | [n] optional generate wr notification (have a _wr_en)
//      | [a] optional generate wr notification (have a _wr_en & _wdata)
// 
// Thus following type of registers:
// uR.W.: Read-write                                              
//      : Value provided by the host. The host can read it and write it.
// u__W.: Write-only                                              
//      : Value provided by the host. The host can only write it.
// u__Wn: Write-only with notification                            
//      : Value provided by the host. The host can only write it.
// kR.__: Read-only register                                      
//      : Value provided by the RTL.
// kRn__: Read-only register with notification  (rd)              
//      : Value provided by the RTL.
// bR.W.: Read-write register                                     
//      : Both the host and the RTL can modify the value. The host can read it and write it.
// bRnWn: Read-write register with notification (rd/wr)           
//      : Both the host and the RTL can modify the value. The host can read it and write it.
// kR.Wa: Read-only register with notification (wr) and action    
//      : Value provided by the RTL. The host can read it. The write data is processed by the RTL.
// kRnWa: Read-only register with notification (rd/wr) and action 
//      : Value provided by the RTL. The host can read it with notify. The write data is processed by the RTL.
// ============================================================================================== //

module {{name}} {% raw %}#({% endraw %}
  parmameter int REG_DATA_W = {{word_size_b}}
  {%- for reg in regs_sv -%}
  {%- if reg.param_snippets != "" -%},
  {{reg.param_snippets}}
  {%- endif -%}
  {%- endfor -%})(
  input  logic                           clk,
  input  logic                           s_rst_n,

  // Axi4 lite Slave Interface sAxi4
  input  logic [AXI4L_ADD_W-1:0]         s_axi4l_awaddr,
  input  logic                           s_axi4l_awvalid,
  output logic                           s_axi4l_awready,
  input  logic [AXI4L_DATA_W-1:0]        s_axi4l_wdata,
  input  logic                           s_axi4l_wvalid,
  output logic                           s_axi4l_wready,
  output logic [1:0]                     s_axi4l_bresp,
  output logic                           s_axi4l_bvalid,
  input  logic                           s_axi4l_bready,
  input  logic [AXI4L_ADD_W-1:0]         s_axi4l_araddr,
  input  logic                           s_axi4l_arvalid,
  output logic                           s_axi4l_arready,
  output logic [AXI4L_DATA_W-1:0]        s_axi4l_rdata,
  output logic [1:0]                     s_axi4l_rresp,
  output logic                           s_axi4l_rvalid,
  input  logic                           s_axi4l_rready
  {%- for reg in regs_sv -%}
  {%- if reg.io_snippets != "" -%}{{reg.io_snippets}}{%- endif -%}
  {%- endfor -%}
);

// ============================================================================================== --
// Axi4l management
// ============================================================================================== --
  logic                    axi4l_awready;
  logic                    axi4l_wready;
  logic [1:0]              axi4l_bresp;
  logic                    axi4l_bvalid;
  logic                    axi4l_arready;
  logic [1:0]              axi4l_rresp;
  logic [AXI4L_DATA_W-1:0] axi4l_rdata;
  logic                    axi4l_rvalid;

  logic                    axi4l_awreadyD;
  logic                    axi4l_wreadyD;
  logic [1:0]              axi4l_brespD;
  logic                    axi4l_bvalidD;
  logic                    axi4l_arreadyD;
  logic [1:0]              axi4l_rrespD;
  logic [AXI4L_DATA_W-1:0] axi4l_rdataD;
  logic                    axi4l_rvalidD;

  logic                    wr_en;
  logic [AXI4L_ADD_W-1:0]  wr_add;
  logic [AXI4L_DATA_W-1:0] wr_data;
  logic                    rd_en;
  logic [AXI4L_ADD_W-1:0]  rd_add;

  logic                    wr_enD;
  logic [AXI4L_ADD_W-1:0]  wr_addD;
  logic [AXI4L_DATA_W-1:0] wr_dataD;
  logic                    rd_enD;
  logic [AXI4L_ADD_W-1:0]  rd_addD;

  //== Local read/write signals
  // Write when address and data are available.
  // Do not accept a new write request when the response
  // of previous request is still pending.
  // Since the ready is sent 1 cycle after the valid,
  // mask the cycle when the ready is r
  assign wr_enD   = (s_axi4l_awvalid & s_axi4l_wvalid
                     & ~(s_axi4l_awready | s_axi4l_wready)
                     & ~(s_axi4l_bvalid & ~s_axi4l_bready));
  assign wr_addD  = s_axi4l_awaddr;
  assign wr_dataD = s_axi4l_wdata;

  // Answer to read request 1 cycle after, when there is no pending read data.
  // Therefore, mask the rd_en during the 2nd cycle.
  assign rd_enD   = (s_axi4l_arvalid
                    & ~s_axi4l_arready
                    & ~(s_axi4l_rvalid & ~s_axi4l_rready));
  assign rd_addD  = s_axi4l_araddr;

  //== AXI4L write ready
  assign axi4l_awreadyD = wr_enD;
  assign axi4l_wreadyD  = wr_enD;

  //== AXI4L read address ready
  assign axi4l_arreadyD = rd_enD;

  //== AXI4L write resp
  logic [1:0]              axi4l_brespD_tmp;
  assign axi4l_bvalidD    = wr_en          ? 1'b1:
                            s_axi4l_bready ? 1'b0 : axi4l_bvalid;
  assign axi4l_brespD     = axi4l_bvalidD ? axi4l_brespD_tmp : '0;
  assign axi4l_brespD_tmp = (wr_add - AXI4L_ADD_OFS) < AXI4L_ADD_RANGE ? AXI4_OKAY : AXI4_SLVERR;

  //== AXI4L read resp
  assign axi4l_rvalidD    = rd_en          ? 1'b1 :
                            s_axi4l_rready ? 1'b0 : axi4l_rvalid;

  always_ff @(posedge clk) begin
    if (!s_rst_n) begin
      axi4l_awready <= 1'b0;
      axi4l_wready  <= 1'b0;
      axi4l_bresp   <= 2'h0;
      axi4l_bvalid  <= 1'b0;

      axi4l_arready <= 1'b0;
      axi4l_rdata   <= 'h0;
      axi4l_rresp   <= 'h0;
      axi4l_rvalid  <= 1'b0;

      wr_en         <= 1'b0;
      rd_en         <= 1'b0;
    end
    else begin
      axi4l_awready <= axi4l_awreadyD;
      axi4l_wready  <= axi4l_wreadyD;
      axi4l_bresp   <= axi4l_brespD;
      axi4l_bvalid  <= axi4l_bvalidD;

      axi4l_arready <= axi4l_arreadyD;
      axi4l_rdata   <= axi4l_rdataD;
      axi4l_rresp   <= axi4l_rrespD;
      axi4l_rvalid  <= axi4l_rvalidD;

      wr_en         <= wr_enD;
      rd_en         <= rd_enD;
    end
  end

  always_ff @(posedge clk) begin
    wr_add  <= wr_addD;
    rd_add  <= rd_addD;
    wr_data <= wr_dataD;
  end

  //= Assignment
  assign s_axi4l_awready = axi4l_awready;
  assign s_axi4l_wready  = axi4l_wready;
  assign s_axi4l_bresp   = axi4l_bresp;
  assign s_axi4l_bvalid  = axi4l_bvalid;
  assign s_axi4l_arready = axi4l_arready;
  assign s_axi4l_rresp   = axi4l_rresp;
  assign s_axi4l_rdata   = axi4l_rdata;
  assign s_axi4l_rvalid  = axi4l_rvalid;

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
    if (axi4l_rvalid) begin
      axi4l_rdataD = s_axi4l_rready ? '0 : axi4l_rdata;
      axi4l_rrespD = s_axi4l_rready ? '0 : axi4l_rresp;
    end
    else begin
      axi4l_rdataD = axi4l_rdata;
      axi4l_rrespD = axi4l_rresp;
      if (rd_en) begin
        axi4l_rrespD = AXI4_SLVERR;
        case(rd_add)
        {%- for reg in regs_sv -%}{{reg.rd_snippets}}{% endfor %}
        endcase // rd_add
      end // if rd_end
    end
  end // always_comb - read

endmodule
