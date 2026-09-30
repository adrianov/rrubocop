user.update(website: 'example.com')
Model.delete(1)
Model.delete_all
Model.destroy_all
FileUtils.touch('file')
::FileUtils.touch('file')
belongs_to(:user).touch(true)
belongs_to(:user).touch(false)
string.insert(0, 'b')
string&.insert(0, 'b')
array.insert(1, :a, :b)
array&.insert!(1, :a, :b)
User.update_all
User.insert
insert(attributes, something_else: true)
insert(attributes, returning: false, something_else: true)
