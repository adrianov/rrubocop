RSpec.describe 'x' do
  it 'bad' do
    expect(5).to eq(price)
           ^ RSpec/ExpectActual: Provide the actual value you are testing to `expect(...)`.
    expect('literal').not_to match(/x/)
           ^^^^^^^^^ RSpec/ExpectActual: Provide the actual value you are testing to `expect(...)`.
  end
end
