module Short
  x = 1
end

module Outer
  class Inner
    x = 1
  end
end

# `class << self` is not a RuboCop namespace module — still counted
module WithSingleton
  class << self
    def a; end
  end
end
