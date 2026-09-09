def start_processing?
  previous_status = take_for_retry!
  return false unless previous_status

  enqueue_processing(previous_status)
  true
end

def foo?
  x == y
end

def ==(other)
  hash == other.hash
end

def initialize
  true
end

def call
  foo == bar
end

# unknown return type — conservative mode skips
def maybe?
  bar
end

def maybe
  bar
end

# Nested def returns must not make the outer method look boolean-only.
def outer
  def inner
    return false
  end
  bar
end
