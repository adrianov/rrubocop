def foo(**config)
  bar(**config)
end

def foo(**options)
  use(options)
  bar(**options)
end

def foo(**options)
  bar(**options.to_h)
end

def foo(**options)
  def inner
    bar(**options)
  end
end

def foo(**options)
end

def foo(**options)
  bar(options:, **options)
end

def foo(**options)
  { **options }
end
