def foo(used, unused)
              ^^^^^^ Lint/UnusedMethodArgument: Unused method argument - `unused`. If it's necessary, use `_` or `_unused` as an argument name to indicate that it won't be used. If it's unnecessary, remove it.
  puts used
end
