{# Template for generating subcase part for read section #}          
{# Warn: Keep indentation in phase with module template (cf. rd_snippets) #}          
          {{ offset }}: begin // register {{ name }}
            axi4l_rrespD = AXI4_OKAY;
            axi4l_rdataD = {{ default }};
          end
