class SomeClass
  def self.class_method
  end
end

module SomeModule
  def self.mod_method
  end
end

class MyClass
  def instance_method
  end
end

# def ClassName.method inside class << self should not be flagged
class Signal
  class << self
    def Signal.trap(sig)
      sig
    end
  end
end
