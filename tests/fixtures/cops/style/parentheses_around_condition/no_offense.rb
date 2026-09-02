if x > 1
  do_something
end

# AllowSafeAssignment: true (default)
if (a = something)
  use(a)
end

while (line = gets)
  process(line)
end

if (result = compute)
  handle(result)
end

# Letter immediately before `(` — RuboCop Parentheses#parens_required?
while(false)
  x = 1
end

begin
  x = 1
end while(false)

# while_post (`begin…end while`) is not registered in RuboCop's on_while
begin
  x = 1
end while (false)
