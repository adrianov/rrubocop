User.update_all(:attr)
     ^^^^^^^^^^ Rails/SkipsModelValidations: Avoid using `update_all` because it skips validations.
User.update_all :name
     ^^^^^^^^^^ Rails/SkipsModelValidations: Avoid using `update_all` because it skips validations.
    others.update_all(current: false)
           ^^^^^^^^^^ Rails/SkipsModelValidations: Avoid using `update_all` because it skips validations.
user&.update_attribute(:website, 'example.com')
      ^^^^^^^^^^^^^^^^ Rails/SkipsModelValidations: Avoid using `update_attribute` because it skips validations.
User.touch
     ^^^^^ Rails/SkipsModelValidations: Avoid using `touch` because it skips validations.
insert(attributes, returning: false)
^^^^^^ Rails/SkipsModelValidations: Avoid using `insert` because it skips validations.
insert(attributes, unique_by: :username)
^^^^^^ Rails/SkipsModelValidations: Avoid using `insert` because it skips validations.
