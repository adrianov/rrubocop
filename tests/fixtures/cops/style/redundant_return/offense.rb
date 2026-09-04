def foo
  return 1
  ^^^^^^ Style/RedundantReturn: Redundant `return` detected.
end

def bar
  return a, b
  ^^^^^^ Style/RedundantReturn: Redundant `return` detected.
end
