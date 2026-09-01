RSpec.describe Foo do
  let(:x) { 1 }
  subject { described_class.new }
  ^^^^^^^ RSpec/LeadingSubject: Declare `subject` above any other `let` declarations.
end
