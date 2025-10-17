# {{regmap.module_name | upper }} documentation
**Date**: {{ now() | date(format="%Y-%m-%d") }}
**Tool Version**: {{ tool_version }}

## RegisterMap Overview

**Module Name**: {{ regmap.module_name }}
**Description**: {{ regmap.description }}
**Offset**: {{ as_hex(val=regmap.offset )}}
**Range**: {{ as_hex(val=regmap.range) }}
**Word Size (b)**: {{ regmap.word_size_b }}
**External Packages**: {%for pkg in regmap.ext_pkg%}"{{pkg}}.sv"{%- if not loop.last %},{% endif -%}{%endfor%}


---

## Section Overview

Below is a summary of all the registers in the current register map:

| Section Name | Offset | Range | Description |
|-------------:|:------:|:-----:|:------------|
{%- for section in regmap.section %}
| [{{ section.name }}](#section-{{ section.name | slugify }}) | {{ as_hex(val=section.offset) }} | {{ as_hex(val=section.range) }} | {{ section.description }} |
{%- endfor %}


---

{% for section in regmap.section %}
## Section {{ section.name | slugify }}

### Register Overview

Below is a summary of all the registers in the current section {{section.name}}:

| Name             | Offset | Access | Description |
|-----------------:|:------:|:------:|:------------|
{%- for register in section.register %}
| [{{ register.name }}](#register-{{ section.name | slugify }}{{ register.name | slugify }}) | {{ as_hex(val=register.offset) }} | {% if register.read_access is containing("Read") %}R{%else%}.{% endif %}{% if register.write_access is containing("Write") %}W{%else%}.{%endif%} |  {{ register.description }} |
{%- endfor %}


---

{% for register in section.register %}
### Register {{ section.name | slugify }}.{{ register.name | slugify }}

- **Description**: {{ register.description }}
- **Owner**: {{ register.owner }}
- **Read Access**: {{ register.read_access }}
- **Write Access**: {{ register.write_access }}
- **Offset**: {{ as_hex(val=register.offset) }}
- **Default**: {%for k,v in register.default %}{%if v is object %}C.f. fields{%else%}{{v}}{%endif%}{%- if not loop.last %}, {% endif -%}{%endfor%}

{% if register.field %}
#### Field Details

Register {{ register.name }} contains following Sub-fields:

| Field Name | Offset_b | Size_b | Default      | Description   |
|-----------:|:--------:|:------:|:------------:|:--------------|
{%- for field in register.field %}
| {{ field.name }}      | {{ field.offset_b }} | {{field.size_b}} | {%- if field.default is object -%} {%for k,v in field.default %}{{v}}{%- if not loop.last %}, {% endif -%}{%endfor%}{% else %} N/A {%-endif-%} | {{ field.description }} |
{%- endfor %}
{% endif %}


---

{% endfor %}{# Register loop #}

{% endfor %}{# Section loop #}
