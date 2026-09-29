block do
^^^^^^^^ Style/ItBlockParameter: Avoid using `it` block parameter for multi-line blocks.
  do_something(it)
end

  list.map do
  ^^^^^^^^^^^ Style/ItBlockParameter: Avoid using `it` block parameter for multi-line blocks.
    it.name
  end

block { do_something(_1) }
                     ^^ Style/ItBlockParameter: Use `it` block parameter.
