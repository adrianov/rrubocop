class Platform
  if RUBY_VERSION >= "3.0"
    def bar
      :modern
    end
  else
    def bar
      :legacy
    end
  end
end

# ActiveSupport::Concern DSL blocks are separate contexts — not duplicates.
module Confirmable
  extend ActiveSupport::Concern

  class_methods do
    def confirmation_enabled?
      true
    end
  end

  included do
    def confirmation_enabled?
      self.class.confirmation_enabled?
    end
  end
end

# self.class_eval is ignored by RuboCop (unlike bare class_eval).
class SelfEval
  self.class_eval do
    def a; 1; end
    def a; 2; end
  end
end
