class C
  def self.build(content, private: false)
    new(value: content, private: private)
  end

  private
  def n
  end
end
