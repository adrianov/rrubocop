items.each { |item| }
^^^^^^^^^^^^^^^^^^^^^ Lint/EmptyBlock: Empty block detected.

items.each do |item|
^^^^^^^^^^^^^^^^^^^^ Lint/EmptyBlock: Empty block detected.
end

Foo.proc {}
^^^^^^^^^^^ Lint/EmptyBlock: Empty block detected.

# `#` inside a string must not count as AllowComments
foo("#") { }
^^^^^^^^^^^^ Lint/EmptyBlock: Empty block detected.
