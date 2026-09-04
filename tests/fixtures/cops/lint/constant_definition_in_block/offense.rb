describe 'x' do
  FOO = 1
  ^^^^^^^ Lint/ConstantDefinitionInBlock: Do not define constants this way within a block.
end

task :lint do
  FILES = []
  ^^^^^^^^^^ Lint/ConstantDefinitionInBlock: Do not define constants this way within a block.
end
