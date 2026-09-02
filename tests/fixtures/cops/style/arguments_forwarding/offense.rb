def foo(*args, &block)
  bar(*args, &block)
  ^^^ Style/ArgumentsForwarding: Use arguments forwarding.
end
