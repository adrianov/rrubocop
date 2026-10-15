RSpec.describe User do
  subject(:user) { described_class.new }

  it "is valid" do
    expect(user.valid?).to be(true)
  end

  it { is_expected.to be_valid }
end

def foo
  it(subject)
end

RSpec.describe User do
  subject(:new_user) { described_class.new }

  shared_examples_for "a valid new user" do
    it "is a User" do
      expect(subject).to be_a(User)
    end

    it "is valid" do
      expect(subject.valid?).to be(true)
    end
  end
end

RSpec.describe User do
  it "uses a local" do
    subject = described_class.new
    expect(subject).to eq(subject)
  end

  it "calls a method" do
    expect(user.subject).to eq(1)
  end

  it "shadows with a block arg" do |subject|
    expect(subject).to eq(1)
  end
end
