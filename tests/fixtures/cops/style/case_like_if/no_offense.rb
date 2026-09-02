if status == :active
  perform_action
else
  check_timeout
end
