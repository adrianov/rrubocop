def foo
  do_something
rescue
^^^^^^ Lint/UselessRescue: Useless `rescue` detected.
  raise
end

def bar
  do_something
rescue => e
  raise e
end
# nitrocop-expect: 9:0 Lint/UselessRescue: Useless `rescue` detected.

def baz
  do_something
rescue
^^^^^^ Lint/UselessRescue: Useless `rescue` detected.
  raise $!
end

raise "TEST_ME" rescue raise rescue nil
# nitrocop-expect: 19:16 Lint/UselessRescue: Useless `rescue` detected.
