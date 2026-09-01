def setup
  key = <<~PRIVATE_KEY
    line
  PRIVATE_KEY
end

def sql
  query = <<SQL
    select 1
SQL
end
