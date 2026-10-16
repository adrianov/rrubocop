def bad
   x = 1
^^^ Layout/IndentationWidth: Use 2 (not 3) spaces for indentation.
end

module Exchange
  module OcoOrders
    class ResolveOrderFields
  Result = Data.define(
  ^^ Layout/IndentationWidth: Use 2 (not -2) spaces for indentation.
    :volume,
  )

  class << self
  ^^ Layout/IndentationWidth: Use 2 (not -2) spaces for indentation.
    def call
      1
    end
  end
    end
  end
end

class Overindented
    def foo
^^^^ Layout/IndentationWidth: Use 2 (not 4) spaces for indentation.
    end
end
