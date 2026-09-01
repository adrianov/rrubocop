foo(
  bar,
  baz
)

# keyword args in parens: blank line between pairs is not "around arguments"
stub_request(:post, 'url').with(
  body: /x/,

  headers: {
    'Accept' => '*/*',
  },
)

# heredoc argument with blank lines inside the heredoc body
expect(client).to have_received(:fly).with(
  msg: <<~MESSAGE,
    **Order failed**

    External order was rejected
  MESSAGE
  level: :error,
  component: 'orders',
)

# receiver and method call on different lines
foo.

  bar(arg)
