def f
  response['error'].tap { |error| raise StandardError, error.inspect if error }
  response
end
