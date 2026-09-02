if status == :active
  perform_action
else
  check_timeout
end

# Below MinBranchesCount (default 3): if + one elsif is allowed.
if kind == :order
  initial_order
elsif kind == :payment
  fixed_payment
else
  unknown
end

if status =~ /active/
  perform_action
elsif status =~ /inactive/
  check_timeout
end
