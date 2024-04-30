{# Template for generating RTL pkg struct #}
{# Warn: Keep indentation in phase with module template (cf. struct_snippets) #}

  typedef struct packed {
    {%- for nos in fields_nos -%}
    logic [({{nos.2}}-1):0] {{nos.0}};
    {% endfor %}
   } {{base_name}}_t;
