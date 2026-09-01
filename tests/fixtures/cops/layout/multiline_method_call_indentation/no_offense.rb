foo = Thing
      .a
      .b
[foo, foo]

shop.product_gold_erp_ids_updated_at
    &.ceil
    &.iso8601

Redis::Regular.instance
              .get('key')
              &.split(',')
              &.uniq

allow(SearchProductsUsingAnyquery)
  .to receive(:call)
  .with(
    'query',
    { shop: 1 }
  )
  .and_call_original

