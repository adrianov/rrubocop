class Foo
  private
  ^^^^^^^ Lint/UselessAccessModifier: Useless `private` access modifier.
end

class AlreadyPublic
  def a; end

  public
  ^^^^^^ Lint/UselessAccessModifier: Useless `public` access modifier.

  def b; end
end

class OnlySingletonAfterPublic
  private
  def hidden; end

  public
  ^^^^^^ Lint/UselessAccessModifier: Useless `public` access modifier.

  def self.commands; end

  private
  ^^^^^^^ Lint/UselessAccessModifier: Useless `private` access modifier.
end
