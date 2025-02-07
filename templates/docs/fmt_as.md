# {{regmap.module_name | upper }} documentation
**Date**: {{ now() | date(format="%Y-%m-%d") }}  
**Tool Version**: {{ tool_version }}  

## RegisterMap Overview

**Module Name**: {{ regmap.module_name }}  
**Description**: {{ regmap.description }}  
**Offset**: {{ regmap.offset }}  
**Range**: {{ regmap.range }}  
**Word Size (b)**: {{ regmap.word_size_b }}  
**External Packages**: {%for pkg in regmap.ext_pkg%}"{{pkg}}.sv"{%- if not loop.last %},{% endif -%}{%endfor%}


---

## Section Overview

Below is a summary of all the registers in the current register map:

| Section Name | Offset | Range | Description |
|-------------:|:------:|:-----:|:------------|
{% for name, section in regmap.section %}
| [{{ name }}](#section-{{ name | slugify }}) | {{section.offset}} | {{section.range}} | {{ section.description }} |
{% endfor %}


---

{% for name, section in regmap.section %}
## Section: {{ name }}

### Register Overview

Below is a summary of all the registers in the current section {{name}}:

| Name             | Offset | Owner    | Read Access | Write Access | Description |
|-----------------:|:------:|:--------:|:-----------:|:------------:|:------------|
{%- for name, register in section.register %}
| [{{ name }}](#register-{{ name | slugify }}) | {{ register.offset }} | {{ register.owner }} | {{ register.read_access }} | {{ register.write_access }} |  {{ register.description }} |
{%- endfor %}


---

{% for name, register in section.register %}
### Register: {{ name }}

- **Description**: {{ register.description }}
- **Owner**: {{ register.owner }}
- **Read Access**: {{ register.read_access }}
- **Write Access**: {{ register.write_access }}
- **Offset**: {{ register.offset }}
- **Default**: {%for k,v in register.default %}{{k}}:{{v}}{%- if not loop.last %}, {% endif -%}{%endfor%} 

{% if register.field %}
#### Field Details

Register {{ name }} contains following Sub-fields:

| Field Name | Offset_b | Size_b | Default      | Description   |
|-----------:|:--------:|:------:|:------------:|:--------------|
{%- for name, field in register.field %}
| {{ name }}      | {{ field.offset_b }} | {{field.size_b}} | {%- if field.default is object -%} {%for k,v in field.default %}{{k}}:{{v}}{%- if not loop.last %}, {% endif -%}{%endfor%}{% else %} N/A {%-endif-%} | {{ field.description }} |
{%- endfor %}
{% endif %}


---

{% endfor %} {# Register loop #}

{% endfor %} {# Section loop #}
