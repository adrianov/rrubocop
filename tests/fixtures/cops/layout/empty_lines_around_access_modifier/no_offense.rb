class Foo
  def bar
  end

  private

  def baz
  end

  protected

  def qux
  end
end

# Access modifier right after class opening (no blank needed before)
class Bar
  private

  def secret
  end
end

# peatio i18n_prefix — protected first in subclass body
class PrefixBackend < Simple
  protected

  def lookup
  end
end

# Access modifier right before end (blank before is enough in around style when after is body end —
# we still require blank after unless at opening; keep blank after here)
class Baz
  def stuff
  end

  private

  def secret
  end
end

# Access modifier as first statement in a block body
Class.new do
  private

  def secret
  end
end
