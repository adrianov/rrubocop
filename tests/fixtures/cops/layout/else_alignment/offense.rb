if foo
  bar
  else
  ^^^^ Layout/ElseAlignment: Align `else` with `if`.
  baz
end

if foo
  bar
  elsif qux
  ^^^^^ Layout/ElseAlignment: Align `elsif` with `if`.
  baz
end

# keyword EndAlignment: else should align with if, not assignment start
_result = if foo
  bar
else
^^^^ Layout/ElseAlignment: Align `else` with `if`.
  baz
end
