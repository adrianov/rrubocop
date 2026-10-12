module Test
  module_function

  def test; end
end

module WithPrivate
  extend self
  def test; end
  private
  def test_private; end
end

module Declarative
  extend self
  def test; end
  private :test
end

module PrivateParens
  extend self
  def test; end
  private()
  def test_private; end
end

class Test
  extend self
  def test; end
end

module Args
  def test; end
  module_function :test
end

module Singleton
  class << self
    def test; end
  end
end

module Receiver
  def test; end
  Other.extend self
end

module Extra
  extend self, Other
  def test; end
end

module InsideMethod
  def test
    extend self
  end
end

module TelemetryMetrics
  EXPORTER = 1

  module_function

  def metric_points(name)
    name
  end
end
