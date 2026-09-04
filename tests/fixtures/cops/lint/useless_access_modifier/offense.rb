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

# ActiveSupport::Concern `class_methods` is not a new scope by default —
# `private` inside it leaves visibility private, so the next `private` is useless.
module ObjectValidation
  class_methods do
    def object_type
    end

    private

    def ensure_empty!
    end
  end

  private
  ^^^^^^^ Lint/UselessAccessModifier: Useless `private` access modifier.

  def validate_object!
  end
end
