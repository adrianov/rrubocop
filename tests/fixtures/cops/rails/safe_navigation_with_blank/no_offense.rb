do_something if foo.blank?
do_something unless foo.blank?
x if foo&.present?
y unless bar.nil?
z = foo&.blank?
foo.blank?
x if (foo&.blank?)
x if foo&.blank? && bar
x if foo&.blank?(1)
while foo&.blank?
  step
end
