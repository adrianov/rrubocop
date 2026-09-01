Rails.application
     .routes
     .url_helpers
     .api_url
     .remove('/x')
     # HACK: trailing period in this comment.
     .gsub('//api', '/api')

foo
  .bar
  .baz
