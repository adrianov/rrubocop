def foo(*args, &block)
  bar(*args, &block)
  ^^^ Style/ArgumentsForwarding: Use arguments forwarding.
end

def foo(*args, &block)
  baz
  bar(*args, &block)
  ^^^ Style/ArgumentsForwarding: Use arguments forwarding.
end

def foo(*args, &block)
  if cond
    bar(*args, &block)
    ^^^ Style/ArgumentsForwarding: Use arguments forwarding.
  end
end

def foo(*args, &block)
  obj.bar *args, &block
  ^^^ Style/ArgumentsForwarding: Use arguments forwarding.
end
