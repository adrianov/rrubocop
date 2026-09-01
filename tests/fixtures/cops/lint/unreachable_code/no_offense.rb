def f
  raise ArgumentError, <<~MSG
    hello
  MSG
end

def g
  return
end
