begin
  foo
rescue
^^^^^^ Style/RescueStandardError: Avoid rescuing without specifying an error class.
  bar
end
