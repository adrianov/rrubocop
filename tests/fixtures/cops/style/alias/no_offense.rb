class C
  alias deletable? retryable?
end

alias ala bala

module M
  def foo
    instance_eval {
      alias bar baz
    }
  end
end

alias $ala $bala

# value used — keep alias_method
public alias_method :a, :b
