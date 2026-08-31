# attr as last statement before end — RuboCop has no right_sibling
module Faraday
  class Env
    attr_reader :request_body
  end
end

class Foo
  attr_accessor :foo

  def do_something
  end
end

class Bar
  attr_accessor :foo
  attr_reader :bar
  attr_writer :baz

  def example
  end
end

class Baz
  attr_accessor :foo
  alias :foo? :foo

  def example
  end
end

class OnlyAttr
  attr_reader :bar
end

# last statement in branch — no right sibling
if condition
  attr_reader :foo
else
  do_something
end
