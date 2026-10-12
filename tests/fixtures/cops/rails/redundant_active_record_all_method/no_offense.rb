User.all

User.all.map(&:do_something)

do_something(User.all)

sum(User.all)

page.all(:parameter)

page.all(:parameter).select(some_filter)

User.all()

User.all().do_something

User.all.find { |item| item.do_something }

User.all.find { _1.do_something }

User.all.find { it.do_something }

User.all.find(&:do_something)

User.all.select { |item| item }

User.all.any? { it.ok? }

User.all&.order(:created_at)

class User
  def do_something
    all.where(id: ids)
  end
end

class User < Foo
  def do_something
    all.where(id: ids)
  end
end

user.articles.all.delete_all

user.articles.all.destroy_all

users = User.all
users.all.delete_all

all.delete_all

def local_all
  all = []
  all.where(id: 1)
end

ActiveSupport::TimeZone.all.first

ActionMailer::Preview.all.first
