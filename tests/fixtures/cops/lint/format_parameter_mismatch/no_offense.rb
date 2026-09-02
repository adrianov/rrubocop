format(
  'notifiers/%<app_name>s/%<name>s',
  app_name: 'a',
  name: 'b'
)

info = [1, 'a', 2]
format('[%4d ms | %s | #%03d] %s', *info, 'x')
format('[%4d ms | %s | #%03d]', *info)
sprintf('%s %s', *args)

format('%*d', 8, 42)
format('%*.*f', 2, 3, 1.5)
format('%5.2f', 1.234)
