_method_name =
  if field.respond_to?(:method_sym) then field.method_sym
  elsif field.respond_to?(:resolver_method) then field.resolver_method
  else
    name.to_sym
  end

if cond
  a
end
