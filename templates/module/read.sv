{# Template for generating subcase part for read section #}          
{# Warn: Keep indentation in phase with module template (cf. rd_snippets) #}          
          {{ offset_cst_name }}: begin // register {{ name }}
            axi4l_rrespD = AXI4_OKAY;
            {% if param_reg %}
            axi4l_rdataD = {{ default_val }};
            {% else %}
            axi4l_rdataD = r_{{name}};
            {% endif %}
          end
