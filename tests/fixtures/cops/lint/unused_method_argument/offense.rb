def foo(used, unused)
              ^^^^^^ Lint/UnusedMethodArgument: Unused method argument - `unused`. If it's necessary, use `_` or `_unused` as an argument name to indicate that it won't be used. If it's unnecessary, remove it.
  puts used
end

# `super()` / `super(x)` do not forward remaining args (not zsuper).
def bare_parens(a)
                ^ Lint/UnusedMethodArgument: Unused method argument - `a`. If it's necessary, use `_` or `_a` as an argument name to indicate that it won't be used. If it's unnecessary, remove it. You can also write as `bare_parens(*)` if you want the method to accept any arguments but don't care about them.
  super()
end
