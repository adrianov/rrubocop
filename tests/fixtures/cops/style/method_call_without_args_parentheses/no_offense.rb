def foo
  super().merge(a: 1)
  bar
end

# CamelCase methods (dry-monads style) keep empty parens.
Success()
Failure()

