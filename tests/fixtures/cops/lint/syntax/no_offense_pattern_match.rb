def call
  each do |cart_item|
    last_price_and_discount(cart_item) =>
      last_price:,
      discount:,
      apihub_discount:,
      price_type:,
      superprice:,
      badge_details: badges_details

    minimal_retail_price = cart_item[:minimal_retail_price]
    [last_price, discount, apihub_discount, price_type, superprice, badges_details, minimal_retail_price]
  end
end
