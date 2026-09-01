# frozen_string_literal: true

class Market < ::Types::BaseObject
  field :trades, [MarketTrade], null: false do
    argument :new_ui_data, Boolean, required: false, default_value: true
  end

  def trades(new_ui_data: true)
    ask = object.ask_unit
    simulator_real_only = ENV.fetch('TRADE_SIMULATOR_ENABLED', false).to_boolean && !new_ui_data
    object.base_market.trades(simulator_real_only:).each_with_object([]) do |trade, tlist|
      trade.symbolize_keys!
      tlist << trade
    end
  end
end
