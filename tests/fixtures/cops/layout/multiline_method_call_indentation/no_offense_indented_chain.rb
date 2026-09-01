ProductCategory
  .joins(:preview_card_positions)
  .distinct
