do_something if foo&.blank?
^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/SafeNavigationWithBlank: Avoid calling `blank?` with the safe navigation operator in conditionals.
bar unless baz&.blank?
^^^^^^^^^^^^^^^^^^^^^^ Rails/SafeNavigationWithBlank: Avoid calling `blank?` with the safe navigation operator in conditionals.
x if obj&.blank?
^^^^^^^^^^^^^^^^ Rails/SafeNavigationWithBlank: Avoid calling `blank?` with the safe navigation operator in conditionals.

def validate_client_trigger_id!
  return unless client_trigger_id&.blank?
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/SafeNavigationWithBlank: Avoid calling `blank?` with the safe navigation operator in conditionals.

  return_error(:client_trigger_id)
end

def assign_trigger_payload!
  if client_trigger_id&.blank?
  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/SafeNavigationWithBlank: Avoid calling `blank?` with the safe navigation operator in conditionals.
    return_error(:client_trigger_id)
  end
end

if ready
  go
elsif foo&.blank?
^^^^^^^^^^^^^^^^^ Rails/SafeNavigationWithBlank: Avoid calling `blank?` with the safe navigation operator in conditionals.
  stop
end

foo&.blank? ? a : b
^^^^^^^^^^^^^^^^^^^ Rails/SafeNavigationWithBlank: Avoid calling `blank?` with the safe navigation operator in conditionals.
