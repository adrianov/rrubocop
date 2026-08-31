{ key: "value" }
{ "string_key" => "value" }
{ 1 => "one" }

# Mixed key types — don't convert symbol rockets
{ "string_key" => "value", :symbol_key => 1 }

# Setter method symbol key — can't use 1.9 syntax
{ :timeouts= => nil }

# Rails routes-style mixed rockets
get '/signout' => 'sessions#destroy', :as => :signout
get '/auth/failure' => 'sessions#failure', :as => :failure
