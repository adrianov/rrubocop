def via_super(**options)
  super(**options)
end

def via_yield(**options)
  yield(**options)
end

def via_hash_arg(**options)
  bar({ **options })
end

def via_index(**options)
  bar[**options]
end
