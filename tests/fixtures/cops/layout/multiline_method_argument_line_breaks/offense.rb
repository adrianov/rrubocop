foo(
  bar, baz,
       ^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
  qux
)

something(
  first, second,
         ^^^^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
  third
)

method_call(
  a, b,
     ^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
  c
)

# Trailing keyword hash pairs sharing a line with positional args
taz("abc",
"foo", "bar", z: "barz",
              ^^^^^^^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
       ^^^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
x: "baz"
)

# Second arg starts on the same line as the end of multiline first arg
taz({
  foo: "edf",
}, "abc")
   ^^^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.

# Hash arg starting on same line as positional arg
taz("abc", {
           ^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
  foo: "edf",
})

# Safe navigation with args on multiple lines
foo&.bar(baz, quux,
              ^^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
  corge)

# Multiple args on first line
do_something(foo, bar, baz,
                       ^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
                  ^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
  quux)

# No-parens command call with keyword args on multiple lines
render json: data, status: :ok,
                   ^^^^^^^^^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
  layout: false

# No-parens command call with positional and keyword args
process records, format: :csv,
                 ^^^^^^^^^^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
  output: path

# Block pass on same line as end of multiline hash arg
def message_area_tag(room, &block)
  tag.div id: "message-area", class: "area", data: {
    controller: "messages",
  }, &block
     ^^^^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
end

# Arguments start on the line after the call
expect(nodes).to(include(
  'kind' => 'OCO', 'cancelId' => wait_oco_take_profit.id.to_s,
                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Layout/MultilineMethodArgumentLineBreaks: Each argument in a multi-line method call must start on a separate line.
  'quantity' => { 'amount' => '0.5' }
))
