begin
  do_something
rescue
^^^^^^ Lint/SuppressedException: Do not suppress exceptions.
end

begin
  do_something
rescue
^^^^^^ Lint/SuppressedException: Do not suppress exceptions.
  ;
end
