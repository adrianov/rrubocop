msg = "hello #{name}"
s = "#{a}#{b}"
plain = "hello"

# Implicit concatenation — RuboCop skips these
joined = "#{a}" \
  "#{b}"

# Hash label keys are not flagged by RuboCop (not dstr)
h = { "#{key}": 1 }
