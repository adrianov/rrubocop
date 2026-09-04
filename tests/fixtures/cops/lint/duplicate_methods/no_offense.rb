class Platform
  if RUBY_VERSION >= "3.0"
    def bar
      :modern
    end
  else
    def bar
      :legacy
    end
  end
end
