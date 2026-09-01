items.each do |x|
  puts x
  end
  ^^^ Layout/BlockAlignment: `end` is not aligned with `do` beginning at column 11.

items.map do |x|
  x * 2
    end
    ^^^ Layout/BlockAlignment: `end` is not aligned with `do` beginning at column 10.

# FN: end aligns with RHS of assignment instead of LHS
answer = prompt.select do |menu|
           menu.choice "A"
         end
         ^^^ Layout/BlockAlignment: `end` is not aligned with `do` beginning at column 23.
