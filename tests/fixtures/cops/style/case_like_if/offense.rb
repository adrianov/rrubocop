if status =~ /active/
^^ Style/CaseLikeIf: Convert `if-elsif` to `case-when`.
  perform_action
elsif status =~ /inactive/
  check_timeout
elsif status =~ /invalid/
  report_invalid
end
