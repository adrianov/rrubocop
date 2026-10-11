User.order(:foo).each
User.limit(1).each
User.select(:id).each
User.lock.each
User.where(foo.order(:x)).each
errors.where(a: 1).each
all.each
users.each
User.all.find_each

class Other
  def call
    all.each { |u| u }
  end
end
