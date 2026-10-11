def nine_times
  9.times { yield }
  ^ Style/ExplicitBlockArgument: Consider using explicit block argument in the surrounding method's signature over `yield`.
end

def with_tmp_dir
  Dir.mktmpdir do |tmp_dir|
    Dir.chdir(tmp_dir) { |dir| yield dir }
    ^ Style/ExplicitBlockArgument: Consider using explicit block argument in the surrounding method's signature over `yield`.
  end
end

def load_variants(group_type)
  measure(operation:, group_type:) { yield }
  ^ Style/ExplicitBlockArgument: Consider using explicit block argument in the surrounding method's signature over `yield`.
end

def call(worker, job, queue)
  run(job, labels) { yield }
  ^ Style/ExplicitBlockArgument: Consider using explicit block argument in the surrounding method's signature over `yield`.
end

def pass_arg
  run { |x| yield x }
  ^ Style/ExplicitBlockArgument: Consider using explicit block argument in the surrounding method's signature over `yield`.
end

def block_do
  9.times do
  ^ Style/ExplicitBlockArgument: Consider using explicit block argument in the surrounding method's signature over `yield`.
    yield
  end
end

def outer
  class << self
    tap { yield }
    ^ Style/ExplicitBlockArgument: Consider using explicit block argument in the surrounding method's signature over `yield`.
  end
end
