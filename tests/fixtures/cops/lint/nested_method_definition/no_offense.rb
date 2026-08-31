def build_fake_cache
  class << fake_cache
    def redis
      klass = Class.new do
        def get(v)
          v
        end
      end
      klass
    end
  end
end

def plain
  1
end
