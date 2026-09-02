def foo
  x = 1
  ^^ Lint/UselessAssignment: Useless assignment to variable - `x`.
  y = 2
  y
end

# Bare `super` must not mark locals used (RuboCop zsuper = method args only).
def with_zsuper(a)
  x = 1
  ^^ Lint/UselessAssignment: Useless assignment to variable - `x`.
  super
end
