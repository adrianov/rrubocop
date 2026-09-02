def foo(used, _unused)
  puts used
end

def empty(x)
end

def nyi(x)
  raise NotImplementedError
end

# Bare `super` forwards all method arguments (RuboCop zsuper).
def initialize(user:, offer_id:)
  @offer_id = offer_id
  super
end

def method_missing(method, *args)
  return nil if method == :skip
  super
end

def respond_to_missing?(method, include_private = false)
  method == :skip || super
end

# Nested bare `super` still forwards the enclosing method's args.
def nested(a)
  tap { super }
end
