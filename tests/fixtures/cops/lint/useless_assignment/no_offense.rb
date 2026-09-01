def scan
  next_byte = peek
  case
  when (token = TABLE[next_byte])
    token
  end
end

def test
  conn = pool.lease
  def conn.requires_reloading?
    true
  end
  conn
end

def singleton_obj
  klass = Class.new
  class << klass
    def x; end
  end
end

def with_binding
  github_user = `whoami`.chomp
  binding
end
