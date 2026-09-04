if true
  1
  end
  ^^^ Layout/EndAlignment: `end` at 3, 2 is not aligned with `if` at 1, 0.

# case as method arg — keyword style: `end` must match `case`, not the call
foo(case x
when 1 then 1
end)
^^^ Layout/EndAlignment: `end` at 8, 0 is not aligned with `case` at 6, 4.
