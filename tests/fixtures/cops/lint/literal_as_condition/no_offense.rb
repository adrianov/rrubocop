# while true is Style/InfiniteLoop, not LiteralAsCondition
while true
  break
end

until false
  break
end

if x
  1
end
