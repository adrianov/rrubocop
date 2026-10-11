User.all.each
         ^^^^ Rails/FindEach: Use `find_each` instead of `each`.

User.where(id: 1).each
                  ^^^^ Rails/FindEach: Use `find_each` instead of `each`.

User.all.each { order }
         ^^^^ Rails/FindEach: Use `find_each` instead of `each`.

class User < ApplicationRecord
  def call
    all.each { |u| u }
        ^^^^ Rails/FindEach: Use `find_each` instead of `each`.
  end
end

class Record < ActiveRecord::Base
  def call
    all.each
        ^^^^ Rails/FindEach: Use `find_each` instead of `each`.
  end
end

class Record < ::ApplicationRecord
  def call
    all.each
        ^^^^ Rails/FindEach: Use `find_each` instead of `each`.
  end
end

::Sidekiq::Queue.all.each do |queue|
                     ^^^^ Rails/FindEach: Use `find_each` instead of `each`.
  queue.name
end
