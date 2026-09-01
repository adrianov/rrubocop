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

# good: method body that is an argument (`memoize def`) — RuboCop aligns
memoize def need_notice?
  shop.present? &&
  kind == 'online' &&
  shop.deleted_at.present?
end

# good: operation inside kwargs is aligned with the first operand
f.input :style, selected: obj.values ||
                          WidgetStyle.default.values

