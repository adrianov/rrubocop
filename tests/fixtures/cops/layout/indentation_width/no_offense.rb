def post_json(destination, body, headers = {})
  post(
    destination,
    params: (
               String === body ? body : body.to_json
             ),
    headers: headers,
  )
end

def ok
  1
end
