RSpec.describe 'x' do
  it 'good' do
    expect([a, b]).to all(be_valid)
  end

  it 'not only expects' do
    [a, b].each do |user|
      setup(user)
      expect(user).to be_valid
    end
  end

  it 'two block args' do
    [[a, 1]].each do |user, n|
      expect(user).to eq(n)
    end
  end

  it 'not_to is not the matcher' do
    [a].each do |user|
      expect(user).not_to be_nil
    end
  end

  it 'parameterless without expect' do
    [a].each { setup(_1) }
  end
end
