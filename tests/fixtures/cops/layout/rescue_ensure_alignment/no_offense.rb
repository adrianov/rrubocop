def truncate
  ClickHouse::Record.connection
                    .tables
                    .each do |table|
    ClickHouse::Record.connection.execute("TRUNCATE #{table}")
  rescue ActiveRecord::ActiveRecordError
    next
  end
end

begin
  work
rescue
  handle
end
