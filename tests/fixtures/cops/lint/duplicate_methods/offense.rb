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
