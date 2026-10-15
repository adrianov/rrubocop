def foo(**options)
        ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  bar(**options)
      ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
end

def foo(m, **kwargs)
           ^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  bar(m, extra: :kwarg, **kwargs)
                        ^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
end

def register(method_name, instrument, **options)
                                      ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  published = "#{name.demodulize.underscore}.#{method_name}"
  var = :"@#{method_name}"
  define_singleton_method(method_name) do
    instance_variable_get(var) || instance_variable_set(var, instrument.build(published, **options))
                                                                                         ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  end
end

def foo(**options)
        ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  bar(**options)
      ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  self.options
  options()
end

def foo(**options)
        ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  x = { **options }
  bar(options: 1, **options)
                  ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
end

def foo(**options)
        ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  bar(**options)
      ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  baz { |options| 1 }
end

def foo(**options)
        ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  super(**options)
        ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
end

def foo(**options)
        ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  yield(**options)
        ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
end

def foo(**options)
        ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  bar({ **options })
        ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
end

def foo(**options)
        ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
  bar[**options]
      ^^^^^^^^^ Style/ArgumentsForwarding: Use anonymous keyword arguments forwarding (`**`).
end
