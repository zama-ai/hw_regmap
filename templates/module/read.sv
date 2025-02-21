{# Template for generating subcase part for read section #}
{# Warn: Keep indentation in phase with module template (cf. rd_snippets) #}
          {{ offset_cst_name }}[AXIL_ADD_RANGE_W-1:0]: begin // register {{ name }}
            {% if param_reg %}
            axil_rdataD = {{name}}_default;
            {% else %}
            axil_rdataD = r_{{name}};
            {% endif %}
          end
