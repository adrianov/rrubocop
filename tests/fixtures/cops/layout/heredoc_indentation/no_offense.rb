def foo
  expect(msg).to eq(
    <<~MSG.chomp
      Errors:
       - login:
         - too short
    MSG
  )
end
