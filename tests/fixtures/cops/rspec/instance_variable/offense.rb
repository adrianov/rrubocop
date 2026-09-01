RSpec.describe Foo do
  before { @user = 1 }
  it 'reads' do
    expect(@user).to eq(1)
           ^^^^^ RSpec/InstanceVariable: Avoid instance variables - use let, a method call, or a local variable (if possible).
  end
end
