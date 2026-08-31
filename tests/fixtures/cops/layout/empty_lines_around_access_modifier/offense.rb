class Foo
  def bar
  end
  private
  ^^^^^^^ Layout/EmptyLinesAroundAccessModifier: Keep a blank line before and after `private`.
  def baz
  end
  protected
  ^^^^^^^^^ Layout/EmptyLinesAroundAccessModifier: Keep a blank line before and after `protected`.
  def qux
  end
end

# Access modifier at class opening missing blank after
class Helper
  protected
  ^^^^^^^^^ Layout/EmptyLinesAroundAccessModifier: Keep a blank line after `protected`.
  def action
  end
end
