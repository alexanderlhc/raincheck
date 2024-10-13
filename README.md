# Raincheck tool

Depends on: https://openweathermap.org/api/one-call-api

Used with HomeAssistant, eg:

```yaml
sequence:
  - action: shell_command.raincheck
    response_variable: raincheck_response
    data: {}
  - variables:
      will_rain: "{{ (raincheck_response.stdout | from_json).raincheck }}"
      details: "{{ (raincheck_response.stdout | from_json).details }}"
```
