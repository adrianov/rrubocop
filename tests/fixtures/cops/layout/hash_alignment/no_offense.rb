{
  host: 'h',
  port: 1, # comment
  tls: false, # another
}

send_file path, type: MIME,
                disposition: 'attachment',
                filename: NAME

# Right-hand pattern matching can misparse in tree-sitter; must stay clean.
Catalog.call(product:,
             shop: @shop) =>
               price_type:,
               discount:,
               superprice:
