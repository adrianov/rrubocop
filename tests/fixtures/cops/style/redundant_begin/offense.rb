def foo
  begin
  ^^^^^ Style/RedundantBegin: Redundant `begin` block detected.
    do_something
  rescue
    handle
  end
end
