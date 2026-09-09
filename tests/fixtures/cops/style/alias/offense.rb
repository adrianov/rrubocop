class C
  alias_method :deletable?, :retryable?
  ^^^^^^^^^^^^ Style/Alias: Use `alias` instead of `alias_method` in a class body.
end

alias_method :ala, :bala
^^^^^^^^^^^^ Style/Alias: Use `alias` instead of `alias_method` at the top level.

alias :foo :bar
      ^^^^^^^^^ Style/Alias: Use `alias foo bar` instead of `alias :foo :bar`.
