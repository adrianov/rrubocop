z = a &&
    b

# good: leading operator on continuation line
def regexp_first_argument?(send_node)
  send_node.first_argument&.regexp_type? \
    && REGEXP_ARGUMENT_METHODS.include?(send_node.method_name)
end

# good: if keyword condition with aligned operands
if a &&
   b
  c
end
