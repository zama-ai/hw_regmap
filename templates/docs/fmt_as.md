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
{%- for sec_name, section in regmap.section %}
| [{{ sec_name }}](#section-{{ sec_name | slugify }}) | {{ as_hex(val=section.offset) }} | {{ as_hex(val=section.range) }} | {{ section.description }} |
{%- endfor %}


---

{% for sec_name, section in regmap.section %}
## Section {{ sec_name | slugify }}

### Register Overview

Below is a summary of all the registers in the current section {{sec_name}}:

| Name             | Offset | Access | Description |
|-----------------:|:------:|:------:|:------------|
{%- for reg_name, register in section.register %}
| [{{ reg_name }}](#register-{{ sec_name | slugify }}.{{ reg_name | slugify }}) | {{ as_hex(val=register.offset) }} | {% if register.read_access is containing("Read") %}R{%else%}.{% endif %}{% if register.write_access is containing("Write") %}W{%else%}.{%endif%} |  {{ register.description }} |
{%- endfor %}


---

{% for reg_name, register in section.register %}
### Register {{ sec_name | slugify }}.{{ reg_name | slugify }}

- **Description**: {{ register.description }}
- **Owner**: {{ register.owner }}
- **Read Access**: {{ register.read_access }}
- **Write Access**: {{ register.write_access }}
- **Offset**: {{ as_hex(val=register.offset) }}
- **Default**: {%for k,v in register.default %}{%if v is object %}C.f. fields{%else%}{{v}}{%endif%}{%- if not loop.last %}, {% endif -%}{%endfor%} 

{% if register.field %}
#### Field Details

Register {{ reg_name }} contains following Sub-fields:

| Field Name | Offset_b | Size_b | Default      | Description   |
|-----------:|:--------:|:------:|:------------:|:--------------|
{%- for field_name, field in register.field %}
| {{ field_name }}      | {{ field.offset_b }} | {{field.size_b}} | {%- if field.default is object -%} {%for k,v in field.default %}{{v}}{%- if not loop.last %}, {% endif -%}{%endfor%}{% else %} N/A {%-endif-%} | {{ field.description }} |
{%- endfor %}
{% endif %}


---

{% endfor %} {# Register loop #}

{% endfor %} {# Section loop #}
