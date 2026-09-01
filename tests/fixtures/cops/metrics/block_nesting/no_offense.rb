def execute
  if print?
    x
  else
    begin
      save
    rescue IOError
      abort
    end
  end
end
