Kernel.puts 'ok'
Rails.logger.info 'ok'

# `p` as block param / local — not a stdout call (tree-sitter identifier).
items.map { |p| p.dig('id') }
items.map { |(p, _), _| p }
def puts
  1
end
foo(puts)
puts.bar

# Assignment target / later local use — not stdout calls.
puts = 1
puts
puts += 1
puts, y = 2, 3

def local_puts
  puts = :shadow
  puts
end

# Method / block / lambda parameters shadow Kernel#puts.
def f(puts)
  puts
end

def g(puts:)
  puts
end

items.each { |puts| puts }
->(puts) { puts }
