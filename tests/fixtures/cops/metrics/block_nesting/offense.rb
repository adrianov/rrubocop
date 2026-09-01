# rrubocop-expect: 5:8 Metrics/BlockNesting: Avoid more than 3 levels of block nesting. [4/3]
def foo
  if a
    if b
      if c
        if d
          if e
            puts e
          end
        end
      end
    end
  end
end
