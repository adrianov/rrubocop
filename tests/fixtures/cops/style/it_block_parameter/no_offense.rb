block { do_something(it) }

block { |arg| do_something(arg) }

block { do_something(_1, _2) }

block { do_something(_2) }

it = 1
block { do_something(it) }

def foo(it)
  block do
    it
  end
end

foo
  .bar { it }
