class Foo
  private

  memoize def client_ip
    request.remote_ip
  end
end
