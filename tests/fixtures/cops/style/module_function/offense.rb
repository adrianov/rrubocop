module Test
  extend self
  ^^^^^^^^^^^ Style/ModuleFunction: Use `module_function` instead of `extend self`.
  def test; end
end

module Only
  extend self
  ^^^^^^^^^^^ Style/ModuleFunction: Use `module_function` instead of `extend self`.
end

module Parens
  extend(self)
  ^^^^^^^^^^^^ Style/ModuleFunction: Use `module_function` instead of `extend self`.
  def test; end
end

module Spaced
  extend  self
  ^^^^^^^^^^^^ Style/ModuleFunction: Use `module_function` instead of `extend self`.
  def test; end
end

module WithProtected
  extend self
  ^^^^^^^^^^^ Style/ModuleFunction: Use `module_function` instead of `extend self`.
  def test; end
  protected
  def other; end
end

module Outer
  extend self
  ^^^^^^^^^^^ Style/ModuleFunction: Use `module_function` instead of `extend self`.
  def a; end

  module Inner
    extend self
    ^^^^^^^^^^^ Style/ModuleFunction: Use `module_function` instead of `extend self`.
    def b; end
  end
end
