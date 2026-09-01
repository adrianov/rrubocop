# frozen_string_literal: true

class Deposits < Grape::API
  params do
    optional :currency, type: String, values: -> { Currency.enabled.codes(bothcase: true) }, desc: -> {
                                                                                                       "Currency value contains #{Currency.enabled.codes(bothcase: true).join(',')}"
                                                                                                     }
  end
end
