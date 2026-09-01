(1..4).reduce(0) do |acc, i|
  next acc if i.odd?
  acc + i
end

(1..4).inject(0) do |acc, i|
  next acc unless i.even?
  acc + i
end

# Nested block: bare next is OK (exits the inner block, not reduce).
(1..4).reduce(0) do |acc, i|
  [1, 2].each do |j|
    next if j == 1
  end
  acc + i
end
