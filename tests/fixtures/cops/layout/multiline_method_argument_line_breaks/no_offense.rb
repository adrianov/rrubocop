foo(
  bar,
  baz,
  qux
)

something(first, second, third)

method_call(
  a,
  b,
  c
)

# All args on same line in multiline call (all_on_same_line? early return)
taz(
  "abc", "foo"
)

# Heredoc argument ends on its opener line; arguments start after the call line
File.write(
  file_path,
  <<~YAML,
    first:
      queue: low
  YAML
)

notify(
  topic: 'alerts',
  msg: <<~MESSAGE,
    Something happened
  MESSAGE
)

# Single keyword hash arg should not trigger
render(
  status: :ok,
  json: payload
)

# Bracket assignment should be skipped
bar['foo'] = ::Time.zone.at(
               huh['foo'],
             )

# Bracket assignment with multiple args on same line
a['b',
    'c', 'd'] = e

# Safe navigation with single arg
foo&.bar(baz)

# Safe navigation with all args on one line
foo&.bar(baz, quux)

# No-parens command call on one line
render json: data, status: :ok

# No-parens command call each arg on separate line
render json: data,
       status: :ok,
       layout: false

# No-parens command call with single keyword arg (no expansion needed)
render status: :ok

# Block pass with all args on one line (no multiline span)
def foo(&)
  bar(a, b, &)
end

# Block pass with keyword args each on separate line
def baz(&)
  tag.div(
    id: "area",
    class: "widget",
    &
  )
end

# Block pass with no-parens keyword args on one line (no multiline span)
def qux(&)
  render json: data, status: :ok, &
end
