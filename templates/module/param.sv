{# Template for generating RTL parameter #}
{# Warn: Keep indentation in phase with module template (cf. param_snippets) #}
{% for d in default_name %}
    parameter int {{ d }} = 0
    {%- if loop.last %}{% break %}{% endif -%},
{% endfor %}
