def with_tmp_dir_ok(&block)
  Dir.mktmpdir do |tmp_dir|
    Dir.chdir(tmp_dir, &block)
  end
end

def extra
  run { yield 1 }
end

def mismatch
  run { |a| yield a, b }
end

def shorter
  run { |a, b| yield a }
end

def body
  run do
    yield
    puts 1
  end
end

9.times { yield }

class Foo
  bar { yield }
end
