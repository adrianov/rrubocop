def foo(*args)
  bar(*args)
end

def foo(*args, &block)
  args
  bar(*args, &block)
end

def foo(*args, &block)
  def inner
    bar(*args, &block)
  end
end
