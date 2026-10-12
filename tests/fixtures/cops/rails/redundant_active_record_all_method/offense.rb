User.all.find(id)
     ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

User.all.order(:created_at)
     ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

users.all.where(id: ids)
      ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

user.articles.all.order(:created_at)
              ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

User.all().order(:created_at)
     ^^^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

User.all.find_each(&:do_something)
     ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

WidgetPosition.all.find_each do |widget_position|
               ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.
end

do_something(User.all.order(:created_at))
                  ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

do_something { User.all.find }
                    ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

User.all.select(:age)
     ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

User.all.delete_all
     ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

User.all.destroy_all
     ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

User.all.first
     ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.

class User < ApplicationRecord
  scope :admins, -> { all.where(admin: true) }
                      ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.
end

class User < ::ApplicationRecord
  scope :admins, -> { all.where(admin: true) }
                      ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.
end

class User < ActiveRecord::Base
  scope :admins, -> { all.where(admin: true) }
                      ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.
end

class User < ::ActiveRecord::Base
  scope :admins, -> { all.where(admin: true) }
                      ^^^ Rails/RedundantActiveRecordAllMethod: Redundant `all` detected.
end
