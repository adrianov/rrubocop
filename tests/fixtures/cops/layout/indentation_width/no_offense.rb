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
