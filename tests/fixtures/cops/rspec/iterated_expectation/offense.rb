RSpec.describe 'x' do
  it 'each expect' do
    [a, b].each do |user|
    ^^^^^ RSpec/IteratedExpectation: Prefer using the `all` matcher instead of iterating over an array.
      expect(user).to be_valid
    end
  end

  it 'multiple expects' do
    [a, b].each do |user|
    ^^^^^ RSpec/IteratedExpectation: Prefer using the `all` matcher instead of iterating over an array.
      expect(user).to be_valid
      expect(user).to be_ok
    end
  end

  it 'numbered parameter' do
    [a, b].each { expect(_1).to be_valid }
    ^^^^^ RSpec/IteratedExpectation: Prefer using the `all` matcher instead of iterating over an array.
  end
end
