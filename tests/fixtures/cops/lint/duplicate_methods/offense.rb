class Foo
  def bar
    1
  end

  def bar
  ^^^^^^^ Lint/DuplicateMethods: Method `#bar` is defined at both test.rb:2 and test.rb:6.
    2
  end
end

class CaseVariant
  case RUBY_VERSION
  when '3.0'
    def bar; 1; end
  when '2.7'
    def bar; 2; end
    ^^^^^^^^^^^^^^^ Lint/DuplicateMethods: Method `#bar` is defined at both test.rb:14 and test.rb:16.
  end
end

class EvalDup
  class_eval do
    def baz; 1; end
    def baz; 2; end
    ^^^^^^^^^^^^^^^ Lint/DuplicateMethods: Method `#baz` is defined at both test.rb:22 and test.rb:23.
  end
end
