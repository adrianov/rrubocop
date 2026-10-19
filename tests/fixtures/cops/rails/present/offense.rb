!x.blank?
^^^^^^^^^ Rails/Present: Use `x.present?` instead of `!x.blank?`.

do_something unless foo.blank?
             ^^^^^^^^^^^^^^^^^ Rails/Present: Use `if foo.present?` instead of `unless foo.blank?`.

x = 1
x unless bar.blank?
  ^^^^^^^^^^^^^^^^^ Rails/Present: Use `if bar.present?` instead of `unless bar.blank?`.

# Multiline lambda with modifier unless — offense at `unless` keyword, not start of expression
updates << -> {
  do_something
} unless user_enabled.blank?
  ^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/Present: Use `if user_enabled.present?` instead of `unless user_enabled.blank?`.

!name.blank?
^^^^^^^^^^^^ Rails/Present: Use `name.present?` instead of `!name.blank?`.

!user.email.blank?
^^^^^^^^^^^^^^^^^^ Rails/Present: Use `user.email.present?` instead of `!user.email.blank?`.

foo && !foo.empty?
^^^^^^^^^^^^^^^^^^ Rails/Present: Use `foo.present?` instead of `foo && !foo.empty?`.

data && !data.empty?
^^^^^^^^^^^^^^^^^^^^ Rails/Present: Use `data.present?` instead of `data && !data.empty?`.

obj.value && !obj.value.empty?
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/Present: Use `obj.value.present?` instead of `obj.value && !obj.value.empty?`.

foo != nil && !foo.empty?
^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/Present: Use `foo.present?` instead of `foo != nil && !foo.empty?`.

record != nil && !record.empty?
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Rails/Present: Use `record.present?` instead of `record != nil && !record.empty?`.

!!foo && !foo.empty?
^^^^^^^^^^^^^^^^^^^^ Rails/Present: Use `foo.present?` instead of `!!foo && !foo.empty?`.
not x.blank?
^^^^^^^^^^^^ Rails/Present: Use `x.present?` instead of `not x.blank?`.
foo unless blank?
    ^^^^^^^^^^^^^ Rails/Present: Use `if present?` instead of `unless blank?`.
!blank?
^^^^^^^ Rails/Present: Use `present?` instead of `!blank?`.
unless foo.blank?
^^^^^^^^^^^^^^^^^ Rails/Present: Use `if foo.present?` instead of `unless foo.blank?`.
  do_something
end
foo and !foo.empty?
^^^^^^^^^^^^^^^^^^^ Rails/Present: Use `foo.present?` instead of `foo and !foo.empty?`.
!foo.nil? && !foo.empty?
^^^^^^^^^^^^^^^^^^^^^^^^ Rails/Present: Use `foo.present?` instead of `!foo.nil? && !foo.empty?`.
!nil? && !empty?
^^^^^^^^^^^^^^^^ Rails/Present: Use `present?` instead of `!nil? && !empty?`.
