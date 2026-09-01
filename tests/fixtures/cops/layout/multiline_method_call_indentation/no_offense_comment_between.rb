def call
  super.to_monad
       # comment
       .fmap(&:to_h)
end
