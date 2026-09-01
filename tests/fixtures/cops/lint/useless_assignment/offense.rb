def foo
  x = 1
  ^^ Lint/UselessAssignment: Useless assignment to variable - `x`.
  y = 2
  y
end
