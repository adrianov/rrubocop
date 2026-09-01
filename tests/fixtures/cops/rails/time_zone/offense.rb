Time.at(1)
     ^^ Rails/TimeZone: Do not use `Time.at` without zone. Use one of `Time.zone.at`, `Time.current`, `Time.at.in_time_zone`, `Time.at.utc`, `Time.at.getlocal`, `Time.at.xmlschema`, `Time.at.iso8601`, `Time.at.jisx0301`, `Time.at.rfc3339`, `Time.at.httpdate`, `Time.at.to_i`, `Time.at.to_f` instead.

Time.now
     ^^^ Rails/TimeZone: Do not use `Time.now` without zone. Use one of `Time.zone.now`, `Time.current`, `Time.now.in_time_zone`, `Time.now.utc`, `Time.now.getlocal`, `Time.now.xmlschema`, `Time.now.iso8601`, `Time.now.jisx0301`, `Time.now.rfc3339`, `Time.now.httpdate`, `Time.now.to_i`, `Time.now.to_f` instead.

"2012-03-02 16:05:37".to_time
                      ^^^^^^^ Rails/TimeZone: Do not use `String#to_time` without zone. Use `Time.zone.parse` instead.

"March".to_time
        ^^^^^^^ Rails/TimeZone: Do not use `String#to_time` without zone. Use `Time.zone.parse` instead.
