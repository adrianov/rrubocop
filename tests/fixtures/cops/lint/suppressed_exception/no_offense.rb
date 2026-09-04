begin
  do_something
rescue => e
  handle(e)
end

begin
  do_something
rescue StandardError
  retry
end
