_x = 'hello'
_y = "it's got a quote"
_z = "has a \n newline"

# Multi-line double-quoted string without interpolation — RuboCop skips
_sql = "SELECT * FROM foo
       WHERE bar = baz"

# Nested strings inside interpolation must not be flagged
_msg = "hello #{data["key"]}"
_log = "value: #{record.dig("a", "b")}"
_out = "#{items.join(", ")}"
_path = "#{Rails.root.join("lib/logstash_patch.rb")}"
