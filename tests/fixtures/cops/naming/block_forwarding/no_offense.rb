def foo(&)
  bar(&)
end

def foo(&block)
  block.call
end

def foo(name:, &block)
  bar(&block)
end
