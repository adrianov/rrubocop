# frozen_string_literal: true

class Deposits < Grape::API
  params do
    optional :currency, type: String, values: -> { Currency.enabled.codes(bothcase: true) }, desc: -> {
                                                                                                       "Currency value contains #{Currency.enabled.codes(bothcase: true).join(',')}"
                                                                                                     }
  end
end

def calc
  c = Account.where("x").group(:y).order("y").
    limit(2).offset(1).sum(:z)
  c
end

# RuboCop on_block checks body against `end`; misaligned end accepts body at end+width
def db_setup_namespaced
  Dir.chdir(".") do
   rails "db:migrate"
 end
end

def process_orders
  OnlineOrder.where(status: :holded)
             .limit(10)
             .find_each(batch_size: 10) do |order|
               update_order(order)
             end
end

def update_scopes
  ProductCategory.actual
                 .roots
                 .find_each do |parent|
    use(parent)
  end
end

def summaries
  ProductReview.annotate('x')
               .group(:id)
               .pluck(:id)
               .to_h do |id|
                 [id, 1]
               end
end

def send_sms
  begin
    deliver
  rescue Vonage::APIError,
         Timeout::Error,
         Net::OpenTimeout
    retry_later
  end
end

